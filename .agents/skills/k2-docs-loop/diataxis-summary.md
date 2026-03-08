# Diataxis Framework Summary

## Tutorial: Learn Diataxis

### What You Will Accomplish
You will understand the four kinds of documentation and be able to classify any documentation page using the Diataxis compass.

### Steps

1. **Start with the core idea**
   - There are exactly four kinds of documentation: tutorials, how-to guides, reference, and explanation
   - These categories come from two dimensions of documentation need

2. **Understand the two dimensions**
   - **Action vs Cognition**: Does the content guide doing or provide knowledge?
   - **Acquisition vs Application**: Does it serve learning or work?

3. **Use the compass to classify**
   - Ask: "Is this action or cognition?"
   - Ask: "Is this acquisition or application?"
   - The intersection gives you the type

4. **Check your understanding**
   - Try classifying 3-5 pages from any documentation site
   - Use the compass table below as your reference

## How-to Guides

### How to Classify Documentation Using Diataxis

**Goal**: Determine the correct Diataxis category for any documentation content

**Prerequisites**: You must be looking at actual documentation content, not planning from scratch

**Steps**:

1. **Identify the content's primary purpose**
   - Read the first two paragraphs
   - Ask: "What is this trying to help me DO or KNOW?"

2. **Apply the compass test**
   - Question 1: Does the content inform ACTION (steps, doing) or COGNITION (facts, thinking)?
   - Question 2: Does it serve ACQUISITION (learning, study) or APPLICATION (work, tasks)?

3. **Determine the category**
   - Action + Acquisition = Tutorial
   - Action + Application = How-to guide
   - Cognition + Application = Reference
   - Cognition + Acquisition = Explanation

4. **Verify your classification**
   - Check if the language matches the category using the type checklist below
   - If substantial content belongs to another category, it may need splitting

### How to Structure Documentation Hierarchy

**Goal**: Organize documentation that serves multiple user types or deployment scenarios

**When to use**: When you have overlapping concerns such as different user groups or deployment environments

**Steps**:

1. **Identify user segments**
   - List the distinct user groups or contexts
   - Verify they have meaningfully different documentation needs

2. **Choose primary dimension**
   - Option A: Diataxis categories at top level, user segments beneath
   - Option B: User segments at top level, Diataxis categories beneath

3. **Evaluate shared content**
   - If content is mostly shared, prefer one arrangement
   - If content is mostly distinct, prefer the other

4. **Create landing pages when the content volume justifies them**
   - For each section, write overview text rather than only lists
   - Group related items into smaller subsections when lists grow too long

## Reference

### The Four Documentation Types

| Type | Purpose | Serves | Content Focus | Answers |
|------|---------|--------|---------------|---------|
| Tutorial | Learning experience | Acquisition of skill | Practical steps under guidance | "Can you teach me to...?" |
| How-to guide | Practical directions | Application of skill | Actions to solve a problem | "How do I...?" |
| Reference | Technical description | Application of skill | Facts about the machinery | "What is...?" |
| Explanation | Discursive treatment | Acquisition of skill | Understanding and context | "Why...?" |

### Compass Decision Table

| If the content... | ...and serves the user's... | ...then it must belong to... |
|-------------------|-----------------------------|------------------------------|
| informs action | acquisition of skill | a tutorial |
| informs action | application of skill | a how-to guide |
| informs cognition | application of skill | reference |
| informs cognition | acquisition of skill | explanation |

### Terminology Mapping

- **Action** = practical steps, doing
- **Cognition** = theoretical knowledge, thinking
- **Acquisition** = study, learning
- **Application** = work, tasks

## Explanation

### Why the Four Types Are Sufficient

The Diataxis framework identifies exactly four documentation types because it maps to the complete territory of human skill development. Two dimensions define documentation needs:

1. **Action/Cognition**: Documentation either guides action or informs cognition

2. **Acquisition/Application**: The user is either acquiring skill or applying skill

These dimensions create four quarters. In the Diataxis model, these quarters define the complete territory of craft documentation.

### When Intuition Fails

The map is reliable but intuition can mislead. Common failure patterns:

- **Tutorial/How-to conflation**: Tutorials teach; how-to guides direct work
- **Reference/Explanation blending**: Explanation creeping into reference material obscures facts
- **Partial collapse**: When boundaries blur, documentation becomes less effective

The compass tool prevents these errors by forcing explicit classification through the two key questions.

### User Cycle Interaction

Users move through documentation types cyclically, but not necessarily in order:

- **Learning phase**: Tutorial
- **Goal phase**: How-to guide
- **Information phase**: Reference
- **Understanding phase**: Explanation

Then the cycle repeats at deeper levels or for new skills.

### Quality Dimensions

**Functional quality**:
- Accuracy, completeness, consistency, precision
- Independent characteristics, objectively measurable

**Deep quality**:
- Feels good to use, has flow, fits human needs
- Interdependent characteristics, subjectively assessed
- Depends on functional quality

Diataxis helps expose functional quality gaps and supports deep quality by structuring documentation around user needs.

---

## LLM Drafting Checklist

### Before Writing Any Page

- [ ] Identify which of the four types this page will be
- [ ] Run the compass test: action/cognition? acquisition/application?
- [ ] Check that the page is dominated by a single type
- [ ] Write the page title to clearly signal the type

### Tutorial Checklist

- [ ] Uses "We will..." not "You will learn..."
- [ ] Starts with concrete, particular tools/materials
- [ ] Provides expected output after every step
- [ ] Contains minimal explanation
- [ ] Has no choices, alternatives, or branches
- [ ] Ends with a meaningful, visible result
- [ ] Could be repeated by a learner for practice

### How-to Guide Checklist

- [ ] Title starts with "How to..." or equivalent
- [ ] Addresses a specific, real-world problem
- [ ] Assumes user already knows what they want to achieve
- [ ] Contains only actions and conditional logic where needed
- [ ] No explanatory digressions, link out if needed
- [ ] Starts and ends at reasonable, meaningful points
- [ ] Prepares for unexpected situations when relevant

### Reference Checklist

- [ ] Structure mirrors the product/code structure
- [ ] Contains only facts, descriptions, and specifications
- [ ] Uses neutral, objective language throughout
- [ ] No opinions, explanations, or instructions
- [ ] Provides examples only for illustration
- [ ] Follows consistent patterns and formats
- [ ] Can be consulted, not read linearly

### Explanation Checklist

- [ ] Title could be prefixed with "About..."
- [ ] Makes connections between concepts
- [ ] Provides context: history, design decisions, constraints
- [ ] Discusses alternatives, trade-offs, perspectives
- [ ] Contains deliberate perspective when relevant
- [ ] Stays within bounded topic
- [ ] Written for reflection away from the product

### Classification Emergency Protocol

If you cannot classify after 30 seconds:
1. Answer: Is this about DOING or KNOWING?
2. Answer: Is this for LEARNING or WORKING?
3. Look up the intersection in the compass table
4. If still unclear, the content is probably mixed and needs splitting
