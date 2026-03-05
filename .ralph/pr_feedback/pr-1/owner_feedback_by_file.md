# PR #1 Owner Feedback By File

Source review: https://github.com/djosh34/pgtuskmaster_rust/pull/1

Filtered to comments by `djosh34` only.

Total inline comments: 82

Files with comments: 14

## `docs/src/operator/configuration.md`
Comment count: 20
Placements: L20, L21, L23, L29, L35, L56, L66, L80, L92, L127, L151, L165, L207, L207, L215, L217, L222, L226, L241, L241

- L20 RIGHT: why does rewind need a source? it should rewind to current leader? i don't see why this is needed
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890052975

- L21 RIGHT: some here
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890053340

- L23 RIGHT: let's not recommend 'prefer'
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890054549

- L29 RIGHT: how is tls disabled, but auth is still tls?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890056468

- L35 RIGHT: security?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890057834

- L56 RIGHT: better examples that are secure
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890059503

- L80 RIGHT: ok it went full overboard with this 'why this exists, tradeoffs, and matters' i need much better docs, and not this cringe
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890064017

- L92 RIGHT: v2? this is greenfield? no way there is an 'old' version: it literally does not exist
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890065915

- L127 RIGHT: etcd auth?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890070094

- L66 RIGHT: pgbackrest config?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890081816

- L151 RIGHT: important to say?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890091455

- L165 RIGHT: why?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890095978

- L207 RIGHT: which file?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890100357

- L207 RIGHT: see what is the point of a file logger if you don't specify a path?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890101812

- L215 RIGHT: just plain NO, i want to provide config yml file and that should work
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890104582

- L217 RIGHT: this is just plain dumb that it is needed
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890105787

- L222 RIGHT: is this also passed to postgres?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890107795

- L226 RIGHT: never deletes? what is the point then?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890110073

- L241 RIGHT: is this good enough?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890114038

- L241 RIGHT: like why only those files? also signal still used in pg16?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890163139


## `docs/src/quick-start/first-run.md`
Comment count: 1
Placements: L1

- L1 RIGHT: missing the quick start config? why not docker stuff (maybe cuz not here yet)
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890194495


## `docs/src/quick-start/index.md`
Comment count: 2
Placements: L3, L11

- L3 RIGHT: incredibly cringe
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890185518

- L11 RIGHT: useless file imho
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890186607


## `docs/src/quick-start/initial-validation.md`
Comment count: 1
Placements: L1

- L1 RIGHT: remove
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890191910


## `docs/src/quick-start/prerequisites.md`
Comment count: 1
Placements: L1

- L1 RIGHT: future task: replace with docker
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890189045


## `docs/src/start-here/problem.md`
Comment count: 1
Placements: L1

- L1 RIGHT: general remark in the docs to prepend numbers like 'xx_[filename].md' to make them have the same order as in the website. also name them the same as on the website
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890179380


## `docs/src/verification/index.md`
Comment count: 1
Placements: L1

- L1 RIGHT: dogshit, remove entirely from docs
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890181450


## `docs/src/verification/task-33-docs-verification-report.md`
Comment count: 1
Placements: L1

- L1 RIGHT: remove, is dogshit
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890182281


## `src/ha/decide.rs`
Comment count: 15
Placements: L38, L44, L55, L73, L84, L88, L105, L128, L142, L303, L348, L379, L384, L389, L565

- L38 RIGHT: abstract these info gathering steps into another function outputting a struct?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890208676

- L44 RIGHT: why not match pattern?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890210914

- L55 RIGHT: same for this, like maybe higher level decide between those, and for each have separate decide flow
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890229991

- L73 RIGHT: wait is this returned? but there is ;
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890232794

- L105 RIGHT: apply?? we must KEEP decide pure. This is NOT pure!
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890240519

- L88 RIGHT: less mut. don't like it
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890244981

- L84 RIGHT: why this? why mut?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890245623

- L128 RIGHT: general remark, please make everything non mut, and fully functional. not sure how to enforce it/lint that. but in general this function must become without any side effects
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890255592

- L142 RIGHT: candidates?!? why doesn't it just directly return its decision? I just don't get this code
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890263260

- L303 RIGHT: this is horrific and unneeded
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890265449

- L348 RIGHT: horrific
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890268496

- L379 RIGHT: code smell....
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890276455

- L384 RIGHT: what is this? why is this mut?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890281954

- L389 RIGHT: we will not allow this. what do you mean you write a 'last_error' here? why not a typed error result like it common in rust?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890290488

- L565 RIGHT: preserve what?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890305693


## `src/ha/e2e_multi_node.rs`
Comment count: 3
Placements: L1, L40, L59

- L40 RIGHT: this should be part of some test config imo
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890310906

- L59 RIGHT: what why?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890314337

- L1 RIGHT: general remark about this file not being inside test/ dir
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890325606


## `src/ha/e2e_partition_chaos.rs`
Comment count: 11
Placements: L18, L83, L163, L249, L318, L331, L375, L380, L527, L720, L758

- L18 RIGHT: this feels partially overlapping with that other e2e file....
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890334649

- L83 RIGHT: why not the same fixture with two variants? not sure if that is better, just asking...
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890338185

- L163 RIGHT: what is this acracadabra code?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890343210

- L249 RIGHT: just like the other test/ files, this should be better divided into multiple files + dirs and all should go into test/  too much mixing here
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890348994

- L318 RIGHT: wasn't there smth like wait for in tokio or smth?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890352728

- L331 RIGHT: let's think about untangling this. Having a poller that checks this stuff during the whole test, i feel like
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890357003

- L375 RIGHT: yeah so instead of having this, we would have something that continuously checks all our invariants during the whole test
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890360166

- L380 RIGHT: horrific code....
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890360920

- L527 RIGHT: there is no sql macro or smth?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890363926

- L720 RIGHT: finalize_partition_scenario? what?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890367941

- L758 RIGHT: this is indeed a 'chaos test' given the absolute chaos this code is......
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890370978


## `src/ha/state.rs`
Comment count: 3
Placements: L96, L113, L156

- L96 RIGHT: don't like this... why the default like this....
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890377094

- L113 RIGHT: also this.. bro what do you think? why is rewind on the same host? that must be inferred from the member keys..... what was the idea here?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890381185

- L156 RIGHT: magic numbers, not from config?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890383483


## `src/ha/worker.rs`
Comment count: 20
Placements: L1, L47, L81, L99, L113, L127, L159, L243, L246, L248, L284, L305, L364, L394, L410, L469, L481, L498, L586, L672

- L1 RIGHT: file is FAR to large....
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890386414

- L47 RIGHT: great!
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890388572

- L81 RIGHT: why does it need a clone tho?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890392984

- L99 RIGHT: after this, what could possibly be happening in this one function
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890396214

- L113 RIGHT: serde json in here?!? WHAT IS HAPPENING
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890398401

- L127 RIGHT: im very confused, it is all state based?  why is there also role, phase? more?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890401664

- L159 RIGHT: dcs should abstract this away, the ha worker must just READ from a struct that doesn't need dealing with any paths
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890407591

- L243 RIGHT: i feel like this entire function could be coded better, but not sure yet how
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890416746

- L246 RIGHT: more something for the job manager no?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890418130

- L248 RIGHT: why can't the process manager have a ref to config, which also contains data_dir?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890420760

- L284 RIGHT: feels very duplicative here, i mean for all actions. isn't there a way for that process manager to just get a ref to config and use from it what's needed? is this a rust limitation?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890426837

- L305 RIGHT: why is the source a 'defaults'  either this is wrong, or the name of that field is very confusing? like you basebackup FROM the leader right?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890430666

- L364 RIGHT: not much other comments to add, other than the main problem i already commented....
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890434778

- L394 RIGHT: take over restored data dir? this should be separate flow i feel like? like this file and in general ha is for the ha loop, not mixing concerns like it does now
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890441701

- L410 RIGHT: encoded? why? 
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890444633

- L469 RIGHT: wipe data dir?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890447829

- L481 RIGHT: this is wrong place for this concern. like why a map of string, object effectively?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890450726

- L498 RIGHT: as in, move to dcs part
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890452024

- L586 RIGHT: same general comment about dcs
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890453827

- L672 RIGHT: why was this needed
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2890907208


## `src/logging/mod.rs`
Comment count: 2
Placements: L243, L264

- L264 RIGHT: what is this? why is this?
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2891081758

- L243 RIGHT: why is this needed
  Link: https://github.com/djosh34/pgtuskmaster_rust/pull/1#discussion_r2891083239


