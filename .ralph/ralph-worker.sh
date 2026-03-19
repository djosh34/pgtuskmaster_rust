#!/bin/bash

# ralph-worker.sh — the actual iteration loop.
# Not meant to be run directly. Use ralph.sh or the ralph-pgtuskmaster systemd service.

set -euo pipefail

DO_TASK_PROMPT="ralph-do-task.md"
CHOOSE_TASK_PROMPT="ralph-choose-task.md"
MODEL_PROFILE_FILE="model.txt"

SOURCE_PATH="${BASH_SOURCE[0]}"
SCRIPT_DIR="$(cd -P "$( dirname "$SOURCE_PATH" )" >/dev/null 2>&1 && pwd)"
echo "Running from $SCRIPT_DIR"


MODE="${1:-${MODE:-claude}}"

echo "Using MODE: $MODE (set MODE=opencode|claude|codex)"


# Create archive directory if it doesn't exist (needed before finding max iteration)
ARCHIVE_DIR="$SCRIPT_DIR/archive"
mkdir -p "$ARCHIVE_DIR"
mkdir -p "$SCRIPT_DIR/progress"

# Note: Progress email is sent directly by progress_append.sh
# It is NOT stopped when ralph-worker exits (intentionally persists)



# Find the highest iteration number from archive directory
# Files are named like: 20260128_210801_iteration_001.log
LAST_ITERATION=$(ls -1 "$ARCHIVE_DIR/"*_iteration_*.log 2>/dev/null | \
  sed -n 's/.*_iteration_\([0-9]*\)\.log/\1/p' | \
  sort -n | tail -1 | sed 's/^0*//' || echo "0")
LAST_ITERATION=${LAST_ITERATION:-0}
START=$((LAST_ITERATION + 1))
echo "Starting from iteration $START (last completed: $LAST_ITERATION)"

# 1. Enable telemetry
export CLAUDE_CODE_ENABLE_TELEMETRY=1

# Codex OpenTelemetry settings are configured in .codex/config.toml.


export OPENCODE_CONFIG="$SCRIPT_DIR/.ralph/opencode_gpt.json"

STOP_FOUND=0

if [[ -f "$SCRIPT_DIR/STOP" ]]; then
  echo ""
  echo ""
  echo ""
  echo ""
  echo "============================================="
  echo "STOP file detected in $SCRIPT_DIR"
  echo "Please remove the STOP file to start iterations."
  echo "============================================="
  exit 1
fi

i=$START

# ---------------------------------------------------------------------------
# Exponential backoff on consecutive failures
#
#   Tier 1: 10 retries, 10 s between each
#   Tier 2: 10 retries, 60 s between each
#   Tier 3: unlimited retries, 600 s between each
#
# On any success the backoff resets to the beginning of tier 1.
# ---------------------------------------------------------------------------
BACKOFF_TIER=1            # current tier (1, 2, or 3)
BACKOFF_COUNT=0           # retries spent in the current tier
TOTAL_FAILURES=0          # total consecutive failures across all tiers
LAST_SUCCESS_EPOCH=$(date +%s)  # epoch of last successful iteration

backoff_delay() {
  case $BACKOFF_TIER in
    1) echo 10  ;;
    2) echo 60  ;;
    *) echo 600 ;;
  esac
}

# Build a human-readable status string for crash emails
backoff_status() {
  local now elapsed mins hrs
  now=$(date +%s)
  elapsed=$((now - LAST_SUCCESS_EPOCH))
  mins=$((elapsed / 60))
  hrs=$((elapsed / 3600))

  if (( hrs > 0 )); then
    printf "Backoff tier %d, attempt %d in tier, %d total failures, %dh %dm since last success" \
      "$BACKOFF_TIER" "$BACKOFF_COUNT" "$TOTAL_FAILURES" "$hrs" "$((mins % 60))"
  elif (( mins > 0 )); then
    printf "Backoff tier %d, attempt %d in tier, %d total failures, %dm %ds since last success" \
      "$BACKOFF_TIER" "$BACKOFF_COUNT" "$TOTAL_FAILURES" "$mins" "$((elapsed % 60))"
  else
    printf "Backoff tier %d, attempt %d in tier, %d total failures, %ds since last success" \
      "$BACKOFF_TIER" "$BACKOFF_COUNT" "$TOTAL_FAILURES" "$elapsed"
  fi
}

backoff_advance() {
  BACKOFF_COUNT=$((BACKOFF_COUNT + 1))
  TOTAL_FAILURES=$((TOTAL_FAILURES + 1))
  local delay
  delay=$(backoff_delay)

  echo "Backoff: tier $BACKOFF_TIER, attempt $BACKOFF_COUNT (sleeping ${delay}s before retry)"

  # Tier 1 & 2 have a cap of 10; tier 3 is unlimited
  if (( BACKOFF_TIER < 3 && BACKOFF_COUNT >= 10 )); then
    BACKOFF_TIER=$((BACKOFF_TIER + 1))
    BACKOFF_COUNT=0
    echo "Backoff: escalated to tier $BACKOFF_TIER"
  fi

  sleep "$delay"
}

backoff_reset() {
  if (( BACKOFF_TIER != 1 || BACKOFF_COUNT != 0 )); then
    echo "Backoff: success — resetting to tier 1"
  fi
  BACKOFF_TIER=1
  BACKOFF_COUNT=0
  TOTAL_FAILURES=0
  LAST_SUCCESS_EPOCH=$(date +%s)
}

# Determine which prompt to use based on current_task.txt existence.
# Returns the prompt filename via PROMPT_NAME global.
PROMPT_SOURCE=""

read_trimmed_first_line() {
  head -n 1 "$1" | tr -d '\r' | xargs
}

determine_prompt() {
  if [[ -f "$SCRIPT_DIR/current_task.txt" ]]; then
    CURRENT_TASK_PATH="$(read_trimmed_first_line "$SCRIPT_DIR/current_task.txt")"
    TEST_CYCLE_TASK_PATH="$(read_trimmed_first_line "$SCRIPT_DIR/test_cycle_task_path.txt")"

    if [[ "$CURRENT_TASK_PATH" == "$TEST_CYCLE_TASK_PATH" ]]; then
      echo "Current task matches test_cycle_task_path.txt; using task file contents as prompt: $CURRENT_TASK_PATH"
      PROMPT_NAME="$CURRENT_TASK_PATH"
      PROMPT_SOURCE="$WORK_DIR/$CURRENT_TASK_PATH"
    elif [[ -f "$SCRIPT_DIR/alt_prompt.txt" ]]; then
      PROMPT_NAME="$(read_trimmed_first_line "$SCRIPT_DIR/alt_prompt.txt")"
      PROMPT_SOURCE="$SCRIPT_DIR/$PROMPT_NAME"
    else
      PROMPT_NAME="$DO_TASK_PROMPT"
      PROMPT_SOURCE="$SCRIPT_DIR/$PROMPT_NAME"
    fi
  elif [[ -f "$SCRIPT_DIR/alt_prompt.txt" ]]; then
    PROMPT_NAME="$(read_trimmed_first_line "$SCRIPT_DIR/alt_prompt.txt")"
    PROMPT_SOURCE="$SCRIPT_DIR/$PROMPT_NAME"
  else
    PROMPT_NAME="$CHOOSE_TASK_PROMPT"
    PROMPT_SOURCE="$SCRIPT_DIR/$PROMPT_NAME"
  fi
#PROMPT_NAME="ralph-stop.md"
}


SHOULD_DO_TEST_NEXT=0

while true; do
  # Capture iteration start timestamp and create output file
  ITERATION_START=$(date +"%Y%m%d_%H%M%S")
  PADDED_NUM=$(printf "%03d" "$i")
  OUTPUT_FILE="$ARCHIVE_DIR/${ITERATION_START}_iteration_${PADDED_NUM}.log"
  RAW_JSON_FILE="$ARCHIVE_DIR/${ITERATION_START}_iteration_${PADDED_NUM}.json"

  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "Iteration $i:"
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  TEST_CYCLE=$(cat "$SCRIPT_DIR/test_cycle.txt" 2>/dev/null || echo "5")

  CODEX_PROFILE="normal_medium"
  if [[ -f "$SCRIPT_DIR/$MODEL_PROFILE_FILE" ]]; then
    CODEX_PROFILE=$(head -n 1 "$SCRIPT_DIR/$MODEL_PROFILE_FILE" | tr -d '\r' | xargs)
    if [[ -z "$CODEX_PROFILE" ]]; then
      CODEX_PROFILE="normal_medium"
      echo "model.txt is empty. Using default codex profile: $CODEX_PROFILE"
    else
      echo "Using codex profile from model.txt: $CODEX_PROFILE"
    fi
  else
    echo "model.txt not found. Using default codex profile: $CODEX_PROFILE"
  fi

  if (( $TEST_CYCLE != 0 )) && (( i % $TEST_CYCLE == 0 )); then
    SHOULD_DO_TEST_NEXT=1
    echo "This is iteration $i, so SHOULD_DO_TEST_NEXT set to 1"
  else
    SHOULD_DO_TEST_NEXT=0
  fi

  # Send update email
  /bin/bash "$SCRIPT_DIR/email.sh" || true

  # Handle task state
  if [[ ! -f "$SCRIPT_DIR/current_task.txt" ]]; then

    if [[ $SHOULD_DO_TEST_NEXT -eq 1 ]]; then
      echo "SHOULD_DO_TEST_NEXT is 1, force next task to be code improvement"
      cp "$SCRIPT_DIR/test_cycle_task_path.txt" "$SCRIPT_DIR/current_task.txt"
      SHOULD_DO_TEST_NEXT=0
      continue
    fi
  fi

  # Determine prompt
  determine_prompt
  echo "                                             "
  echo "                                             "
  echo "Using prompt: $PROMPT_NAME"
  echo "Prompt source: $PROMPT_SOURCE"
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "


  # Write iteration header to file
  {
    echo "============================================="
    echo "Iteration $i"
    echo "Started: $ITERATION_START"
    echo "============================================="
    echo ""
    echo ""
  } > "$OUTPUT_FILE"

  # Function to process output lines
  process_line() {
    printf "\n\n\n"
    printf "\n\n"
    printf "\n\n"
    printf "===================="
    printf "===================="
    printf "===================="
    printf "===================="
    echo "$1" | yq -C -p=json '.'
    printf "===================="
    printf "===================="
    printf "===================="
    printf "===================="
    printf "\n\n"
    printf "\n\n"
    printf "\n\n"

    # Append yq output to log file
    {
      echo ""
      echo ""
      echo ""
      echo "$1" | yq -p=json '.'
      echo ""
      echo ""
    } >> "$OUTPUT_FILE"

    # Append raw JSON line to json file
    echo "$1" >> "$RAW_JSON_FILE"
  }

  ITERATION_ERRORED=0
  if [[ "$MODE" == "opencode" ]]; then
    opencode run "$(cat "$PROMPT_SOURCE")" --model "github-copilot/gpt-5.3-codex" --format json | while read -r line; do
      process_line "$line"
    done || {
      EXIT_CODE=$?
      LAST_LINES=$(tail -600 "$OUTPUT_FILE" 2>/dev/null || echo "<no output>")
      echo "opencode exited with code $EXIT_CODE — continuing to next iteration"
      /bin/bash "$SCRIPT_DIR/email_crash.sh" "$EXIT_CODE" "$LAST_LINES" "$(backoff_status)" || true
      ITERATION_ERRORED=1
    }
  elif [[ "$MODE" == "codex" ]]; then
    codex exec - \
      --dangerously-bypass-approvals-and-sandbox \
      --json \
      --profile "$CODEX_PROFILE" \
      --skip-git-repo-check \
      < "$PROMPT_SOURCE" | while read -r line; do
      process_line "$line"
    done || {
      EXIT_CODE=$?
      LAST_LINES=$(tail -600 "$OUTPUT_FILE" 2>/dev/null || echo "<no output>")
      echo "codex exited with code $EXIT_CODE — continuing to next iteration"
      /bin/bash "$SCRIPT_DIR/email_crash.sh" "$EXIT_CODE" "$LAST_LINES" "$(backoff_status)" || true
      ITERATION_ERRORED=1
    }
  elif [[ "$MODE" == "claude" ]]; then
    claude -p "$(cat "$PROMPT_SOURCE")" --dangerously-skip-permissions --output-format stream-json --verbose --model opus | while read -r line; do
      # Check for STOP file after claude command completes
      if [[ -f "$SCRIPT_DIR/STOP" ]]; then
        echo ""
        echo ""
        echo ""
        echo ""
        echo "============================================="
        echo "STOP file detected in $SCRIPT_DIR"
        echo "Stopping after iteration $i (not continuing to next iteration)"
        echo "============================================="
        echo ""
        echo ""
        echo ""
        echo ""
        STOP_FOUND=1
      fi

      process_line "$line"
    done || {
      EXIT_CODE=$?
      LAST_LINES=$(tail -600 "$OUTPUT_FILE" 2>/dev/null || echo "<no output>")
      echo "claude exited with code $EXIT_CODE — continuing to next iteration"
      /bin/bash "$SCRIPT_DIR/email_crash.sh" "$EXIT_CODE" "$LAST_LINES" "$(backoff_status)" || true
      ITERATION_ERRORED=1
    }
  else
    echo "Unsupported MODE: $MODE"
    echo "Supported modes: opencode, claude, codex"
    exit 2
  fi

  if (( ITERATION_ERRORED )); then
    backoff_advance
  else
    backoff_reset
  fi

  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "============================================="
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "
  echo "                                             "

  echo "Output saved to: $OUTPUT_FILE"

  # Check for STOP file after claude command completes
  if [[ -f "$SCRIPT_DIR/STOP" ]]; then
    echo ""
    echo ""
    echo ""
    echo ""
    echo "============================================="
    echo "STOP file detected in $SCRIPT_DIR"
    echo "Stopping after iteration $i (not continuing to next iteration)"
    echo "============================================="
    exit 1
  fi

  ((i++))
done
