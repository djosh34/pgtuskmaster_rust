You are drafting exactly one documentation file.

Rules:
- Follow Diataxis strictly.
- Use only the supplied repo facts and supplied Diataxis summary.
- If a fact is missing, say "missing source support" instead of inventing.
- ASCII only.
- No em dashes.
- Add diagrams where deemed fitting

Behavior requirements:
- Read the target path and infer the intended page boundary from it.
- Use the Diataxis type that best matches the supplied target and evidence.
- Keep unsupported claims out of the document.
- If an important fact is missing, write "missing source support" at the exact point where that fact would otherwise be needed.

Follow Diataxis method, write one real page, and include diagrams when needed using the syntax:

[diagram about x, y showing relation between z and a, **more details on diagram**]


# target docs path

docs/src/explanation/failure-modes.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/explanation/ha-decision-engine.md
docs/src/explanation/introduction.md
docs/src/explanation/overview.md
docs/src/explanation/process-management.md
docs/src/explanation/trust-model.md
docs/src/how-to/add-cluster-node.md
docs/src/how-to/backup-and-restore.md
docs/src/how-to/bootstrap-cluster.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/configure-tls-security.md
docs/src/how-to/configure-tls.md
docs/src/how-to/debug-cluster-issues.md
docs/src/how-to/handle-complex-failures.md
docs/src/how-to/handle-network-partition.md
docs/src/how-to/handle-primary-failure.md
docs/src/how-to/monitor-via-metrics.md
docs/src/how-to/overview.md
docs/src/how-to/perform-switchover.md
docs/src/how-to/remove-cluster-node.md
docs/src/how-to/run-tests.md
docs/src/overview.md
docs/src/reference/code-organization.md
docs/src/reference/dcs-state-model.md
docs/src/reference/ha-decisions.md
docs/src/reference/http-api.md
docs/src/reference/overview.md
docs/src/reference/pgtm-cli.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/runtime-configuration.md
docs/src/reference/tls-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md
docs/src/tutorial/overview.md
docs/src/tutorial/performing-switchover.md
docs/src/tutorial/validating-cluster-behavior.md


# current docs summary context

===== docs/src/SUMMARY.md =====
# Summary

- [Overview](overview.md)

# Tutorials
- [Tutorials](tutorial/overview.md)
    - [First HA Cluster](tutorial/first-ha-cluster.md)
    - [Observing a Failover Event](tutorial/observing-failover.md)
    - [Performing a Planned Switchover](tutorial/performing-switchover.md)
    - [Validating Cluster Behavior](tutorial/validating-cluster-behavior.md)

# How-To

- [How-To](how-to/overview.md)
    - [Bootstrap a New Cluster from Zero State](how-to/bootstrap-cluster.md)
    - [Check Cluster Health](how-to/check-cluster-health.md)
    - [Add a Cluster Node](how-to/add-cluster-node.md)
    - [Perform Backup and Restore Operations](how-to/backup-and-restore.md)
    - [Configure TLS](how-to/configure-tls.md)
    - [Configure TLS Security](how-to/configure-tls-security.md)
    - [Debug Cluster Issues](how-to/debug-cluster-issues.md)
    - [Handle Complex Failures](how-to/handle-complex-failures.md)
    - [Handle a Network Partition](how-to/handle-network-partition.md)
    - [Handle Primary Failure](how-to/handle-primary-failure.md)
    - [Monitor via CLI Signals](how-to/monitor-via-metrics.md)
    - [Remove a Cluster Node](how-to/remove-cluster-node.md)
    - [Perform a Planned Switchover](how-to/perform-switchover.md)
    - [Run The Test Suite](how-to/run-tests.md)

# Explanation

- [Explanation](explanation/overview.md)
    - [Introduction](explanation/introduction.md)
    - [Architecture](explanation/architecture.md)
    - [Failure Modes and Recovery Behavior](explanation/failure-modes.md)
    - [HA Decision Engine](explanation/ha-decision-engine.md)
    - [Process Management and Execution Domain](explanation/process-management.md)
    - [Trust Model and DCS Coordination Modes](explanation/trust-model.md)

# Reference

- [Reference](reference/overview.md)
    - [Code Organization Reference](reference/code-organization.md)
    - [HTTP API](reference/http-api.md)
    - [HA Decisions](reference/ha-decisions.md)
    - [DCS State Model](reference/dcs-state-model.md)
    - [pgtm CLI](reference/pgtm-cli.md)
    - [pgtuskmaster CLI](reference/pgtuskmaster-cli.md)
    - [Runtime Configuration](reference/runtime-configuration.md)
    - [TLS Configuration Reference](reference/tls-configuration.md)



# diataxis summary markdown

# Diátaxis Framework: Comprehensive Reference Document

## Introduction and Overview

Diátaxis is a systematic approach to technical documentation authoring that identifies four distinct documentation needs and their corresponding forms. The name derives from Ancient Greek δῐᾰ́τᾰξῐς: "dia" (across) and "taxis" (arrangement). It solves problems related to documentation content (what to write), style (how to write it), and architecture (how to organise it).

The framework is pragmatic and lightweight, designed to be easy to grasp and straightforward to apply without imposing implementation constraints. It is built upon the principle that documentation must serve the needs of its users, specifically practitioners in a domain of skill. Diátaxis has been proven in practice and adopted successfully in hundreds of documentation projects, including major organizations like Vonage, Gatsby, and Cloudflare.

### Core Premise

Documentation serves practitioners in a craft. A craft contains both action (practical knowledge, knowing how, what we do) and cognition (theoretical knowledge, knowing that, what we think). Similarly, a practitioner must both acquire and apply their craft. These two dimensions create four distinct territories, each representing a specific user need that documentation must address.

## The Four Kinds of Documentation

### Tutorials

**Definition and Purpose**: A tutorial is an experience that takes place under the guidance of a tutor and is always learning-oriented. It is a practical activity where the student learns by doing something meaningful towards an achievable goal. Tutorials serve the user's acquisition of skills and knowledge—their study—not to help them get something done, but to help them learn. The user learns through what they do, not because someone has tried to teach them.

**Key Characteristics**:
- Tutorials are lessons that take a student by the hand through a learning experience
- They introduce, educate, and lead the user
- Answer the question: "Can you teach me to...?"
- Oriented to learning
- Form: a lesson
- Analogy: teaching a child how to cook

**Essential Obligations of the Teacher**:
The tutorial creator must realize that nearly all responsibility falls upon the teacher. The teacher is responsible for what the pupil is to learn, what the pupil will do to learn it, and for the pupil's success. The pupil's only responsibility is to be attentive and follow directions. The exercise must be meaningful, successful, logical, and usefully complete.

**Key Principles for Writing Tutorials**:

1. **Show the learner where they'll be going**: Provide a picture of what will be achieved from the start to help set expectations and allow the learner to see themselves building towards the completed goal.

2. **Deliver visible results early and often**: Each step should produce a comprehensible result, however small, to help the learner make connections between causes and effects.

3. **Maintain a narrative of the expected**: Keep providing feedback that the learner is on the right path. Show example output or exact expected output. Flag likely signs of going wrong. Prepare the user for possibly surprising actions.

4. **Point out what the learner should notice**: Learning requires reflection and observation. Close the loops of learning by pointing things out as the lesson moves along.

5. **Target the feeling of doing**: The accomplished practitioner experiences a joined-up purpose, action, thinking, and result that flows in a confident rhythm. The tutorial must tie together purpose and action to cradle this feeling.

6. **Encourage and permit repetition**: Learners will return to exercises that give them success. Repetition is key to establishing the feeling of doing.

7. **Ruthlessly minimise explanation**: A tutorial is not the place for explanation. Users are focused on following directions and getting results. Explanation distracts from learning. Provide minimal explanation and link to extended discussions for later.

8. **Ignore options and alternatives**: Guidance must remain focused on what's required to reach the conclusion. Everything else can be left for another time.

9. **Aspire to perfect reliability**: The tutorial must inspire confidence. At every stage, when the learner follows directions, they must see the promised result. A learner who doesn't get expected results quickly loses confidence.

10. **Focus on concrete and particular**: Maintain focus on this problem, this action, this result, leading the learner from step to concrete step. General patterns emerge naturally from concrete examples.

**Language Patterns**:
- "We ..." (first-person plural affirms tutor-learner relationship)
- "In this tutorial, we will ..." (describe what the learner will accomplish)
- "First, do x. Now, do y. Now that you have done y, do z." (no ambiguity)
- "We must always do x before we do y because..." (minimal explanation, link to details)
- "The output should look something like ..." (clear expectations)
- "Notice that ... Remember that ... Let's check ..." (orientation clues)
- "You have built a secure, three-layer hylomorphic stasis engine..." (admire accomplishment)

**Challenges and Difficulties**: Tutorials are rarely done well because they are genuinely difficult to create. The product often evolves rapidly, requiring constant updates. Unlike other documentation where changes can be made discretely, tutorials often require cascading changes through the entire learning journey. The creator must distinguish between what is to be learned and what is to be done, devising a meaningful journey that delivers all required knowledge.

**Food and Cooking Analogy**: Teaching a child to cook demonstrates tutorial principles. The value lies not in the culinary outcome but what the child gains. Success is measured by acquired knowledge and skills, not by whether the child can repeat the process independently. The lesson might be framed around a particular dish, but what the child actually learns are fundamentals like washing hands, holding a knife, understanding heat, timing, and measurement. The child learns through activities done alongside the teacher, not from explanations. Children's short attention spans mean lessons often end before completion, but as long as the child achieved something and enjoyed it, learning has occurred.

### How-to Guides

**Definition and Purpose**: How-to guides are directions that guide the reader through a problem or towards a result. They are goal-oriented and help the user get something done correctly and safely by guiding the user's action. They're concerned with work—navigating from one side to the other of a real-world problem-field.

**Key Characteristics**:
- How-to guides guide the reader
- Answer the question: "How do I...?"
- Oriented to goals
- Purpose: to help achieve a particular goal
- Form: a series of steps
- Analogy: a recipe in a cookery book

**Essential Nature**: A how-to guide addresses a real-world goal or problem by providing practical directions to help the user who is in that situation. It assumes the user is already competent and expects them to use the guide to help them get work done. The guide's purpose is to help the already-competent user perform a particular task correctly. It serves the user's work, not their study.

**Key Principles**:

1. **Address real-world complexity**: A how-to guide must be adaptable to real-world use-cases. It cannot be useless except for exactly the narrow case addressed. Find ways to remain open to possibilities so users can adapt guidance to their needs.

2. **Omit the unnecessary**: Practical usability is more helpful than completeness. Unlike tutorials that must be complete end-to-end guides, how-to guides should start and end in reasonable, meaningful places, requiring readers to join it to their own work.

3. **Provide a set of instructions**: A how-to guide describes an executable solution to a real-world problem. It's a contract: if you're facing this situation, you can work through it by taking the steps outlined. Steps are actions, which include physical acts, thinking, and judgment.

4. **Describe a logical sequence**: The fundamental structure is a sequence implying logical ordering in time. Sometimes ordering is imposed by necessity (step two requires step one). Sometimes it's more subtle—operations might be possible in either order, but one helps set up the environment or thinking for the other.

5. **Seek flow**: Ground sequences in patterns of user activities and thinking so the guide acquires smooth progress. Flow means successfully understanding the user. Pay attention to what you're asking the user to think about and how their thinking flows from subject to subject. Action has pace and rhythm. Badly-judged pace or disrupted rhythm damage flow. At its best, how-to documentation anticipates the user.

6. **Pay attention to naming**: Choose titles that say exactly what the guide shows. Good: "How to integrate application performance monitoring." Bad: "Integrating application performance monitoring" (maybe it's about deciding whether to). Very bad: "Application performance monitoring" (could be about how, whether, or what it is). Good titles help both humans and search engines.

**What How-to Guides Are NOT**: How-to guides are wholly distinct from tutorials, though often confused. Solving a problem cannot always be reduced to a procedure. Real-world problems don't always offer linear solutions. Sequences sometimes need to fork and overlap with multiple entry and exit points. How-to guides often require users to rely on their judgment.

**Language Patterns**:
- "This guide shows you how to..." (describe the problem or task)
- "If you want x, do y. To achieve w, do z." (conditional imperatives)
- "Refer to the x reference guide for a full list of options." (don't pollute with completeness)

**Food and Cooking Analogy**: A recipe is an excellent model. A recipe clearly defines what will be achieved and addresses a specific question ("How do I make...?" or "What can I make with...?"). It's not the responsibility of a recipe to teach you how to make something. A professional chef who has made the same thing many times may still follow a recipe to ensure correctness. Following a recipe requires at least basic competence—someone who has never cooked should not be expected to succeed with a recipe alone. A good recipe follows a well-established format that excludes both teaching and discussion, focusing only on "how" to make the dish.

### Reference

**Definition and Purpose**: Reference guides are technical descriptions of the machinery and how to operate it. Reference material is information-oriented and contains propositional or theoretical knowledge that a user looks to in their work. The only purpose is to describe, as succinctly as possible and in an orderly way. Reference material is led by the product it describes, not by user needs.

**Key Characteristics**:
- Reference guides state, describe, and inform
- Answer the question: "What is...?"
- Oriented to information
- Purpose: to describe the machinery
- Form: dry description
- Analogy: information on the back of a food packet

**Essential Nature**: Reference material describes the machinery in an austere manner. One hardly "reads" reference material; one "consults" it. There should be no doubt or ambiguity—it must be wholly authoritative. Reference material is like a map: it tells you what you need to know about the territory without having to check the territory yourself.

**Key Principles**:

1. **Describe and only describe**: Neutral description is the key imperative. It's not natural to describe something neutrally. The temptation is to explain, instruct, discuss, or opine, but these run counter to reference needs. Instead, link to how-to guides and explanations.

2. **Adopt standard patterns**: Reference material is useful when consistent. Place material where users expect to find it, in familiar formats. Reference is not the place to delight readers with extensive vocabulary or multiple styles.

3. **Respect the structure of the machinery**: The way a map corresponds to territory helps us navigate. Similarly, documentation structure should mirror product structure so users can work through both simultaneously. This doesn't mean forcing unnatural structures, but the logical, conceptual arrangement within code should help make sense of documentation.

4. **Provide examples**: Examples are valuable for illustration while avoiding distraction from description. An example of command usage can succinctly illustrate context without falling into explanation.

**Language Patterns**:
- "Django's default logging configuration inherits Python's defaults. It's available as `django.utils.log.DEFAULT_LOGGING` and defined in `django/utils/log.py`" (state facts about machinery)
- "Sub-commands are: a, b, c, d, e, f." (list commands, options, operations, features, flags, limitations, error messages)
- "You must use a. You must not apply b unless c. Never d." (provide warnings)

**Food and Cooking Analogy**: Checking information on a food packet helps make decisions. When seeking facts, you don't want opinions, speculation, instructions, or interpretation. You expect standard presentation so you can quickly find nutritional properties, storage instructions, ingredients, health implications. You expect reliability: "May contain traces of wheat" or "Net weight: 1000g". You won't find recipes or marketing claims mixed with this information—that could be dangerous. The presentation is so important it's usually governed by law, and the same seriousness should apply to all reference documentation.

### Explanation

**Definition and Purpose**: Explanation is a discursive treatment of a subject that permits reflection. It is understanding-oriented and deepens/broadens the reader's understanding by bringing clarity, light, and context. The concept of reflection is important—reflection occurs after something else, depends on something else, yet brings something new to the subject matter. Its perspective is higher and wider than other types.

**Key Characteristics**:
- Explanation guides explain, clarify, and discuss
- Answer the question: "Why...?"
- Oriented to understanding
- Purpose: to illuminate a topic
- Form: discursive explanation
- Analogy: an article on culinary social history

**Essential Nature**: For the user, explanation joins things together. It's an answer to: "Can you tell me about...?" It's documentation that makes sense to read while away from the product itself (the only kind that might make sense to read in the bath). It serves the user's study (like tutorials) but belongs to theoretical knowledge (like reference).

**The Value and Place of Explanation**:
Explanation is characterized by distance from active concerns. It's sometimes seen as less important, but this is a mistake—it may be less urgent but is no less important; it's not a luxury. No practitioner can afford to be without understanding of their craft. Explanation helps weave together understanding. Without it, knowledge is loose, fragmented, fragile, and exercise of craft is anxious.

**Alternative Names**: Explanation documentation doesn't need to be called "Explanation." Alternatives include Discussion, Background, Conceptual guides, or Topics.

**Key Principles**:

1. **Make connections**: Help weave a web of understanding by connecting to other things, even outside the immediate topic.

2. **Provide context**: Explain why things are so—design decisions, historical reasons, technical constraints. Draw implications and mention specific examples.

3. **Talk about the subject**: Explanation guides are about a topic in the sense of being around it. Names should reflect this—you should be able to place an implicit (or explicit) "about" in front of each title: "About user authentication" or "About database connection policies."

4. **Admit opinion and perspective**: All human activity is invested with opinion, beliefs, and thoughts. Explanation must consider alternatives, counter-examples, or multiple approaches. You're opening up the topic for consideration, not giving instruction or describing facts.

5. **Keep explanation closely bounded**: One risk is that explanation absorbs other things. Writers feel the urge to include instruction or technical description, but documentation already has other places for these. Allowing them to creep in interferes with explanation and removes material from correct locations.

**Language Patterns**:
- "The reason for x is because historically, y..." (explain)
- "W is better than z, because..." (offer judgments and opinions)
- "An x in system y is analogous to a w in system z. However..." (provide context)
- "Some users prefer w (because z). This can be a good approach, but..." (weigh alternatives)
- "An x interacts with a y as follows: ..." (unfold internal secrets)

**Food and Cooking Analogy**: In 1984, Harold McGee published "On Food and Cooking." The book doesn't teach how to cook, doesn't contain recipes (except as historical examples), and isn't reference. Instead, it places food and cooking in context of history, society, science, and technology. It explains why we do what we do in the kitchen and how that has changed. It's not a book to read while cooking, but when reflecting on cooking. It illuminates the subject from multiple perspectives. After reading, understanding is changed—knowledge is richer and deeper. What is learned may or may not be immediately applicable, but it changes how you think about the craft and affects practice.

## Theoretical Foundations

### Two Dimensions of Craft

Diátaxis is based on understanding that a skill or craft contains both action (practical knowledge, knowing how) and cognition (theoretical knowledge, knowing that). These are completely bound up with each other but are counterparts—wholly distinct aspects of the same thing.

Similarly, a practitioner must both acquire and apply their craft. Being "at work" (applying skill) and being "at study" (acquiring skill) are counterparts, distinct but bound together.

### The Map of the Territory

These two dimensions can be laid out on a map—a complete map of the territory of craft. This is a complete map: there are only two dimensions, and they don't just cover the entire territory, they define it. This is why there are necessarily four quarters, and there could not be three or five. It is not an arbitrary number.

### Serving Needs

The map of craft territory gives us the familiar Diátaxis map of documentation. The map answers: what must documentation do to align with these qualities of skill, and to what need is it oriented in each case?

The four needs are:
1. **Learning**: addressed in tutorials, where the user acquires their craft, and documentation informs action
2. **Goals**: addressed in how-to guides, where the user applies their craft, and documentation informs action
3. **Information**: addressed in reference, where the user applies their craft, and documentation informs cognition
4. **Understanding**: addressed in explanation, where the user acquires their craft, and documentation informs cognition

### Why Four and Only Four

The Diátaxis map is memorable and approachable, but reliable only if it adequately describes reality. Diátaxis is underpinned by systematic description and analysis of generalized user needs. This is why the four types are a complete enumeration of documentation serving practitioners. There is simply no other territory to cover.

## The Map and Compass

### The Map

Diátaxis is effective because it describes a two-dimensional structure rather than a list. It places documentation forms into relationships, each occupying a space in mental territory, with boundaries highlighting distinctions.

**Structure Problems**: When documentation fails to attain good structure, architectural faults infect and undermine content. Without clear architecture, creators structure work around product features, leading to inconsistency. Any orderly attempt to organize into clear content types helps, but authors often find content that fails to fit well within categories.

**Expectations and Guidance**: The Diátaxis structure provides clear expectations (to the reader) and guidance (to the author). It clarifies purpose, specifies writing style, and shows placement.

**Blur and Collapse**: There's natural affinity between neighboring forms and a tendency to blur distinctions. When allowed to blur, documentation bleeds into inappropriate forms, causing structural problems that make maintenance harder. In the worst case, tutorials and how-to guides collapse into each other, making it impossible to meet needs served by either.

**Journey Around the Map**: Diátaxis helps documentation better serve users in their cycle of interaction. While users don't literally encounter types in order (tutorials > how-to > reference > explanation), there is sense and meaning to this ordering corresponding to how people become expert. The learning-oriented phase involves diving in under guidance. The goal-oriented phase puts skill to work. The information-oriented phase requires consulting reference. The explanation-oriented phase reflects away from work. Then the cycle repeats.

### The Compass

The compass is a truth-table or decision-tree reducing a complex two-dimensional problem to simpler parts, providing a course-correction tool. It can be applied to user situations needing documentation or to documentation itself that needs moving or improving.

**Using the Compass**: Ask two questions—action or cognition? acquisition or application? The compass yields the answer.

**Table of Contents**:
- If content informs action and serves acquisition of skill → tutorial
- If content informs action and serves application of skill → how-to guide
- If content informs cognition and serves application of skill → reference
- If content informs cognition and serves acquisition of skill → explanation

**Application**: The compass is particularly effective when you're troubled by doubt or difficulty. It forces reconsideration. Use its terms flexibly:
- action: practical steps, doing
- cognition: theoretical knowledge, thinking
- acquisition: study
- application: work

Use the questions in different ways: "Do I think I am writing for x or y?" "Is this writing engaged in x or y?" "Does the user need x or y?" "Do I want to x or y?" Apply them at sentence/ word level or at entire document level.

## Practical Application

### Workflow

Diátaxis is both a guide to content and to documentation process. Most people must make decisions about how to work as they work. Documentation is usually an ongoing project that evolves with the product. Diátaxis provides an approach that discourages planning and top-down workflows, preferring small, responsive iterations from which patterns emerge.

**Use Diátaxis as a Guide, Not a Plan**: Diátaxis describes a complete picture, but its structure is not a plan to complete. It's a guide, a map to check you're in the right place and going in the right direction. It provides tools to assess documentation, identify problems, and judge improvements.

**Don't Worry About Structure**: Don't spend energy trying to get structure correct. If you follow Diátaxis prompts, documentation will assume Diátaxis structure—but because it has been improved, not the other way around. Getting started doesn't require dividing documentation into four sections. Certainly don't create empty structures for each category with nothing in them.

**Work One Step at a Time**: Diátaxis prescribes structure, but whatever the state of existing documentation—even a complete mess—you can improve it iteratively. Avoid completing large tranches before publishing. Every step in the right direction is worth publishing immediately. Don't work on the big picture. Diátaxis guides small steps; keep taking small steps.

**Just Do Something**: 

1. **Choose something**: Any piece of documentation. If you don't have something specific, look at what's in front of you—the file you're in, the last page you read. If nothing, choose literally at random.

2. **Assess it**: Consider it critically, preferably a small thing (page, paragraph, sentence). Challenge it according to Diátaxis standards: What user need is represented? How well does it serve that need? What can be added, moved, removed, or changed to serve that need better? Do language and logic meet mode requirements?

3. **Decide what to do**: Based on answers, decide what single next action will produce immediate improvement.

4. **Do it**: Complete that single action and consider it completed—publish or commit it. Don't feel you need anything else.

This cycle reduces the paralysis of deciding what to do, keeps work flowing, and expends no energy on planning.

**Allow Organic Development**: Documentation should be as complex as it needs to be but no more. A good model is well-formed organic growth that adapts to external conditions. Growth takes place at the cellular level. The organism's structure is guaranteed by healthy cell development according to appropriate rules, not by imposed structure. Similarly, documentation attains healthy structure when internal components are well-formed, building from the inside-out, one cell at a time.

**Complete, Not Finished**: Like a plant, documentation is never finished—it can always develop further. But at every stage, it should be complete—never missing something, always in a state appropriate to its development stage. Documentation should be complete: useful, appropriate to its current stage, in a healthy structural state, and ready for the next stage.

## Complex Documentation Scenarios

### Basic Structure

Application is straightforward when product boundaries are clear:

```
Home                      <- landing page
    Tutorial              <- landing page
        Part 1
        Part 2
        Part 3
    How-to guides         <- landing page
        Install
        Deploy
        Scale
    Reference             <- landing page
        Command-line tool
        Available endpoints
        API
    Explanation           <- landing page
        Best practice recommendations
        Security overview
        Performance
```

Each landing page contains an overview. After a while, grouping within sections might be wise by adding hierarchy:

```
Home                      <- landing page
    Tutorial              <- landing page
        Part 1
        Part 2
        Part 3
    How-to guides         <- landing page
        Install           <- landing page
            Local installation
            Docker
            Virtual machine
            Linux container
        Deploy
        Scale
```

### Contents Pages

Contents pages (home page and landing pages) provide overview of material. There's an art to creating good contents pages; user experience deserves careful consideration.

**The Problem of Lists**: Lists longer than a few items are hard to read unless they have mechanical order (numerical or alphabetical). Seven items seems a comfortable general limit. If you have longer lists, find ways to break them into smaller ones. What matters most is the reader's experience.

**Overviews and Introductory Text**: Landing page content should read like an overview, not just present lists. Remember you're authoring for humans, not fulfilling scheme demands. Headings and snippets catch the eye and provide context. For example, a how-to landing page should have introductory text for each section grouping.

### Two-Dimensional Problems

A more difficult problem occurs when Diátaxis structure meets another structure—often topic areas within documentation or different user types.

**Examples**:
- Product used on land, sea, and air, used differently in each case
- Documentation addressing users, developers building around the product, and contributors maintaining it
- Product deployable on different public clouds with different workflows, commands, APIs, constraints

These scenarios present two-dimensional problems. You could structure by Diátaxis first, then by audience:

```
tutorial
    for users on land
    for users at sea
    for users in the air
[and so on for how-to, reference, explanation]
```

Or by audience first, then Diátaxis:

```
for users on land
    tutorial
    how-to guides
    reference
    explanation
for users at sea
    [...]
```

Both approaches have repetition. What about material that can be shared?

**Understanding the Problem**: The problem isn't limited to Diátaxis—it exists in any documentation system. However, Diátaxis reveals and brings it into focus. A common misunderstanding is seeing Diátaxis as four boxes into which documentation must be placed. Instead, Diátaxis should be understood as an approach, a way of working that identifies four needs to author and structure documentation effectively.

**User-First Thinking**: Diátaxis is underpinned by attention to user needs. We must document the product as it is for the user, as it is in their hands and minds. If the product on land, sea, and air is effectively three different products for three different users, let that be the starting point. If documentation must meet needs of users, developers, and contributors, consider how they see the product. Perhaps developers need understanding of how it's used, and contributors need what developers know. Then be freer with structure, allowing developer-facing content to follow user-facing material in some parts while separating contributor material completely.

**Let Documentation Be Complex**: Documentation should be as complex as it needs to be. Even complex structures can be straightforward to navigate if logical and incorporate patterns fitting user needs.

## Quality Theory

Diátaxis is an approach to quality in documentation. "Quality" is a word in danger of losing meaning—we all approve of it but rarely describe it rigorously. We can point to examples and identify lapses, suggesting we have a useful grasp of quality.

### Functional Quality

Documentation must meet standards of accuracy, completeness, consistency, usefulness, precision. These are aspects of functional quality. A failure in any one means failing a key function. These properties are independent—documentation can be accurate without complete, complete but inaccurate, or accurate, complete, consistent, and useless.

Attaining functional quality means meeting high, objectively-measurable standards consistently across multiple independent dimensions. It requires discipline, attention to detail, and technical skill. Any failure is readily apparent to users.

### Deep Quality

**Characteristics**:
- Feeling good to use
- Having flow
- Fitting human needs
- Being beautiful
- Anticipating the user

Unlike functional quality, these are interdependent. They cannot be checked or measured but can be identified. They are assessed against human needs, not against the world. Deep quality is conditional upon functional quality—documentation cannot have deep quality without being accurate, complete, and consistent. No user will experience it as beautiful if it's inaccurate.

Functional quality presents as constraints—each is a test or challenge we might fail, requiring constant vigilance. Deep quality represents liberation—the work of creativity or taste. To attain functional quality we must conform to constraints; to attain deep quality we must invent.

**How We Recognize Deep Quality**: Consider clothing quality. Clothes must have functional quality (warmth, durability), which is objectively measurable. But quality of materials or workmanship requires understanding clothing. Being able to judge that an item hangs well or moves well requires developing an eye. Yet even without expertise, anyone can recognize excellent clothing because it feels good to wear—your body knows it. Similarly, good documentation feels good; you feel pleasure and satisfaction using it.

### Diátaxis and Quality

Diátaxis cannot address functional quality—it's concerned only with certain aspects of deep quality. However, it can serve functional quality by exposing lapses. Applying Diátaxis to existing documentation often makes previously obscured problems apparent. For example, recommending that reference architecture reflect code architecture makes gaps more visible. Moving explanatory verbiage out of a tutorial often highlights where readers have been left to work things out themselves.

In deep quality, Diátaxis can do more. It helps documentation fit user needs by describing modes based on them. It preserves flow by preventing disruption (like explanation interrupting a how-to guide). However, Diátaxis can never be all that's required for deep quality. It doesn't make documentation beautiful by itself. It offers principles, not a formula. It cannot bypass skills of user experience, interaction design, or visual design. Using Diátaxis does not guarantee deep quality, but it lays down conditions for the possibility of deep quality.

## Distinguishing Documentation Types

### Tutorials vs. How-to Guides

The most common conflation in software documentation is between tutorials and how-to guides. They are similar in being practical guides containing directions to follow. Both set out steps, promise success if followed, and require hands-on interaction.

**What Matters**: The distinction comes from user needs. Sometimes the user is at study, sometimes at work. A tutorial serves study needs—its obligation is to provide a successful learning experience. A how-to guide serves work needs—its obligation is to help accomplish a task. These are completely different needs.

**Medical Example**: Learning to suture a wound in medical school is a tutorial—it's a lesson safely in an instructor's hands. An appendectomy clinical manual is a how-to guide—it guides already-competent practitioners safely through a task. The manual isn't there to teach; it's there to serve work.

**Key Distinctions**:
- Tutorial purpose: help pupil acquire basic competence vs. How-to guide purpose: help already-competent user perform a task
- Tutorial provides learning experience vs. How-to guide directs user's work
- Tutorial follows carefully-managed path vs. How-to guide path can't be managed (real world)
- Tutorial familiarizes learner with tools vs. How-to guide assumes familiarity
- Tutorial takes place in contrived setting vs. How-to guide applies to real world
- Tutorial eliminates unexpected vs. How-to guide prepares for unexpected
- Tutorial follows single line without choices vs. How-to guide forks and branches
- Tutorial must be safe vs. How-to guide cannot promise safety
- In tutorial, responsibility lies with teacher vs. In how-to guide, user has responsibility
- Tutorial learner may not have competence to ask questions vs. How-to guide user asks right questions
- Tutorial is explicit about basic things vs. How-to guide relies on implicit knowledge
- Tutorial is concrete and particular vs. How-to guide is general
- Tutorial teaches general skills vs. How-to guide user completes particular task

**Not Basic vs. Advanced**: How-to guides can cover basic or well-known procedures. Tutorials can present complex or advanced material. The difference is the need served: study vs. work.

### Reference vs. Explanation

Both belong to the theory half of the Diátaxis map—they contain theoretical knowledge, not steps.

**Mostly Straightforward**: Most of the time it's clear which you're dealing with. Reference is well understood from early education. A tidal chart is clearly reference; an article explaining why there are tides is explanation.

**Rules of Thumb**:
- If it's boring and unmemorable, it's probably reference
- Lists and tables generally belong in reference
- If you can imagine reading it in the bath, it's probably explanation
- Asking a friend "Can you tell me more about <topic>?" yields explanation

**Work vs. Study Test**: The real test is: would someone turn to this while working (executing a task) or while stepping away from work to think about it? Reference helps apply knowledge while working. Explanation helps acquire knowledge during study.

**Dangers**: While writing reference that becomes expansive, it's tempting to develop examples into explanation (showing why, what if, or how it came to be). This results in explanatory material sprinkled into reference, which is bad for both—reference is interrupted, and explanation can't develop appropriately.

## Getting Started and Resources

### Quick Start

You don't need to read everything or wait to understand Diátaxis before applying it. In fact, you won't understand it until you start using it. As soon as you have an idea worth applying, try it. Come back to documentation when you need clarity or reassurance. Iterate between work and reflecting on work.

**The Five-Minute Version**:
1. Learn the four kinds: tutorials, how-to guides, reference, explanation
2. Understand the Diátaxis map showing relationships
3. Use the compass (action/cognition? acquisition/application?) to guide decisions
4. Follow the workflow: consider what you see, ask if it could be improved, decide on one small improvement, do it, repeat
5. Do what you like with Diátaxis—it's pragmatic, no exam required. Use what seems worthwhile

### The Website and Community

Diátaxis is the work of Daniele Procida (https://vurt.eu). It has been developed over years and continues to be elaborated. The original context was software product documentation. In 2021, a Fellowship of the Software Sustainability Institute explored its application in scientific research. More recent exploration includes internal corporate documentation, organizational management, education, and application at scale.

**Contact**: Email Daniele at daniele@vurt.org. He enjoys hearing about experiences and reads everything, though can't promise to respond to every message due to volume. For discussion with other users, see the #diataxis channel on the Write the Docs Slack group or the Discussions section of the GitHub repository for the website.

**Citation**: To cite Diátaxis, refer to the website diataxis.fr. The Git repository contains a CITATION.cff file. APA and BibTeX metadata are available from the "Cite this repository" option. You can submit pull requests for improvements or file issues.

**Website**: Built with Sphinx and hosted on Read the Docs, using a modified version of Pradyun Gedam's Furo theme.

### Applying Diátaxis

The pages concerning application are for putting Diátaxis into practice. Diátaxis is underpinned by systematic theoretical principles, but understanding them isn't necessary for effective use. Most key principles can be grasped intuitively. Don't wait to understand before practicing—you won't understand until you start using it.

The core is the four kinds of documentation. If encountering Diátaxis for the first time, start with these. Once you've begun, tools and methods will help smooth your way: the compass, and the workflow (how-to-use-diataxis).

---

Missing source support: None. All requested information is available in the provided Diátaxis source files.


# project manifests and docs config

===== Cargo.toml =====
[package]
name = "pgtuskmaster_rust"
version = "0.1.0"
edition = "2021"

[workspace]
members = ["crates/pgtm_log_derive", "crates/pgtuskmaster_test_support"]

[features]
default = []
internal-test-support = []

[dependencies]
pgtm_log_derive = { path = "crates/pgtm_log_derive" }
clap = { version = "4.5.47", features = ["derive", "env"] }
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
sha2 = "0.10.9"
thiserror = "2.0.12"
tokio = { version = "1.44.1", features = ["sync", "rt", "rt-multi-thread", "macros", "time", "process", "net", "io-util", "fs", "signal"] }
tokio-postgres = "0.7.13"
toml = "0.8.20"
axum = { version = "0.8.6", features = ["http1", "http2", "json", "tokio"] }
axum-server = { version = "0.8.0", features = ["tls-rustls"] }
etcd-client = { version = "0.14.1", features = ["tls"] }
reqwest = { version = "0.12.24", default-features = false, features = ["blocking", "json", "rustls-tls"] }
rustls = { version = "0.23.28", features = ["ring"] }
rustls-pemfile = "2.2.0"
tower = { version = "0.5.2", features = ["util"] }
tower-http = { version = "0.6.2", features = ["trace"] }
tracing = "0.1.41"
tracing-subscriber = "0.3.20"
x509-parser = "0.18.0"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[dev-dependencies]
cucumber = "0.22.1"
futures = "0.3.31"
pgtuskmaster_test_support = { path = "crates/pgtuskmaster_test_support" }
rcgen = "0.14.5"


===== docs/book.toml =====
[book]
authors = ["Joshua Azimullah"]
language = "en"
multilingual = false
src = "src"
title = "pgtuskmaster"

[preprocessor.mermaid]
command = "mdbook-mermaid"

[output]

[output.html]
additional-js = ["mermaid.min.js", "mermaid-init.js"]




# src and test file listing

# src and test file listing

src/api/controller.rs
src/api/mod.rs
src/api/startup.rs
src/api/worker.rs
src/bin/pgtm.rs
src/bin/pgtuskmaster.rs
src/cli/args.rs
src/cli/client.rs
src/cli/config.rs
src/cli/connect.rs
src/cli/error.rs
src/cli/mod.rs
src/cli/output.rs
src/cli/status.rs
src/cli/switchover.rs
src/command/mod.rs
src/config/defaults.rs
src/config/endpoint.rs
src/config/materialize.rs
src/config/mod.rs
src/config/parser.rs
src/config/schema.rs
src/dcs/command.rs
src/dcs/log_event.rs
src/dcs/mod.rs
src/dcs/startup.rs
src/dcs/state.rs
src/dcs/worker.rs
src/dev_support/api.rs
src/dev_support/auth.rs
src/dev_support/binaries.rs
src/dev_support/etcd3.rs
src/dev_support/mod.rs
src/dev_support/namespace.rs
src/dev_support/pg16.rs
src/dev_support/ports.rs
src/dev_support/provenance.rs
src/dev_support/runtime_config.rs
src/dev_support/signals.rs
src/dev_support/tls.rs
src/ha/decide.rs
src/ha/mod.rs
src/ha/process_dispatch.rs
src/ha/reconcile.rs
src/ha/startup.rs
src/ha/state.rs
src/ha/types.rs
src/ha/worker.rs
src/lib.rs
src/logging/core/mod.rs
src/logging/core/queued_record.rs
src/logging/core/runtime.rs
src/logging/event.rs
src/logging/mod.rs
src/logging/postgres_ingest.rs
src/logging/tailer.rs
src/pginfo/conninfo.rs
src/pginfo/log_event.rs
src/pginfo/mod.rs
src/pginfo/query.rs
src/pginfo/startup.rs
src/pginfo/state.rs
src/pginfo/worker.rs
src/postgres_managed.rs
src/postgres_managed_conf.rs
src/postgres_roles.rs
src/process/jobs.rs
src/process/log_event.rs
src/process/mod.rs
src/process/postmaster.rs
src/process/source.rs
src/process/startup.rs
src/process/state.rs
src/process/worker.rs
src/runtime/log_event.rs
src/runtime/mod.rs
src/runtime/node.rs
src/state/coordination.rs
src/state/errors.rs
src/state/ids.rs
src/state/mod.rs
src/state/net.rs
src/state/time.rs
src/state/watch_state.rs
src/tls.rs
tests/AGENTS.md
tests/bdd_api_http.rs
tests/bdd_state_watch.rs
tests/cli_binary.rs
tests/docker/entrypoint.sh
tests/docker/wrappers/pg_basebackup
tests/docker/wrappers/pg_rewind
tests/docker/wrappers/postgres
tests/ha.rs
tests/ha/features/ha_operator_switchovers/ha_operator_switchovers.feature
tests/ha/features/ha_primary_faults_fail_over_then_recover/ha_primary_faults_fail_over_then_recover.feature
tests/ha/features/ha_quorum_loss_and_dcs_loss/ha_quorum_loss_and_dcs_loss.feature
tests/ha/features/ha_rejoin_and_restart_recovery/ha_rejoin_and_restart_recovery.feature
tests/ha/features/ha_replica_faults_keep_cluster_healthy/ha_replica_faults_keep_cluster_healthy.feature
tests/ha/givens/compose/three_node_shared_single.yml
tests/ha/givens/compose/three_node_three_etcd.yml
tests/ha/givens/three_node_shared/configs/pg_hba.conf
tests/ha/givens/three_node_shared/configs/pg_ident.conf
tests/ha/givens/three_node_shared/configs/tls/ca.crt
tests/ha/givens/three_node_shared/configs/tls/node-a.crt
tests/ha/givens/three_node_shared/configs/tls/node-a.key
tests/ha/givens/three_node_shared/configs/tls/node-b.crt
tests/ha/givens/three_node_shared/configs/tls/node-b.key
tests/ha/givens/three_node_shared/configs/tls/node-c.crt
tests/ha/givens/three_node_shared/configs/tls/node-c.key
tests/ha/givens/three_node_shared/configs/tls/observer.crt
tests/ha/givens/three_node_shared/configs/tls/observer.key
tests/ha/givens/three_node_shared/secrets/api-admin-token
tests/ha/givens/three_node_shared/secrets/api-read-token
tests/ha/givens/three_node_shared/secrets/postgres-superuser-password
tests/ha/givens/three_node_shared/secrets/replicator-password
tests/ha/givens/three_node_shared/secrets/rewinder-password
tests/ha/harness.toml
tests/ha/runs/.gitignore
tests/ha/support/config.rs
tests/ha/support/docker/cli.rs
tests/ha/support/docker/mod.rs
tests/ha/support/docker/ryuk.rs
tests/ha/support/error.rs
tests/ha/support/faults/mod.rs
tests/ha/support/givens/mod.rs
tests/ha/support/invariant.rs
tests/ha/support/mod.rs
tests/ha/support/observer/mod.rs
tests/ha/support/observer/pgtm.rs
tests/ha/support/observer/sql.rs
tests/ha/support/process/mod.rs
tests/ha/support/runner/mod.rs
tests/ha/support/steps/mod.rs
tests/ha/support/timeouts/mod.rs
tests/ha/support/topology.rs
tests/ha/support/world/mod.rs
tests/nextest_config_contract.rs


# docker and docs support file listing

docker/Dockerfile
docker/certs/ca.crt
docker/certs/ca.srl
docker/certs/node-a/tls.crt
docker/certs/node-a/tls.key
docker/certs/node-b/tls.crt
docker/certs/node-b/tls.key
docker/certs/node-c/tls.crt
docker/certs/node-c/tls.key
docker/certs/pgtm-client/tls.crt
docker/certs/pgtm-client/tls.key
docker/certs/pgtm/tls.crt
docker/certs/pgtm/tls.key
docker/compose.yml
docker/docker-compose.node.yml
docker/node-a.toml
docker/node-b.toml
docker/node-c.toml
docker/pg/pg_hba.conf
docker/pg/pg_ident.conf
docker/pgtm.toml
docs/book.toml
docs/draft/docs/src/explanation/failure-modes.md
docs/draft/docs/src/explanation/process-management.md
docs/draft/docs/src/explanation/trust-model.md
docs/examples/docker-cluster-node-a.toml
docs/examples/docker-cluster-node-b.toml
docs/examples/docker-cluster-node-c.toml
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/explanation/ha-decision-engine.md
docs/src/explanation/introduction.md
docs/src/explanation/overview.md
docs/src/explanation/process-management.md
docs/src/explanation/trust-model.md
docs/src/how-to/add-cluster-node.md
docs/src/how-to/backup-and-restore.md
docs/src/how-to/bootstrap-cluster.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/configure-tls-security.md
docs/src/how-to/configure-tls.md
docs/src/how-to/debug-cluster-issues.md
docs/src/how-to/handle-complex-failures.md
docs/src/how-to/handle-network-partition.md
docs/src/how-to/handle-primary-failure.md
docs/src/how-to/monitor-via-metrics.md
docs/src/how-to/overview.md
docs/src/how-to/perform-switchover.md
docs/src/how-to/remove-cluster-node.md
docs/src/how-to/run-tests.md
docs/src/overview.md
docs/src/reference/code-organization.md
docs/src/reference/dcs-state-model.md
docs/src/reference/ha-decisions.md
docs/src/reference/http-api.md
docs/src/reference/overview.md
docs/src/reference/pgtm-cli.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/runtime-configuration.md
docs/src/reference/tls-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md
docs/src/tutorial/overview.md
docs/src/tutorial/performing-switchover.md
docs/src/tutorial/validating-cluster-behavior.md
docs/tmp/docs/src/explanation/failure-modes.prompt.md
docs/tmp/docs/src/explanation/process-management.prompt.md
docs/tmp/docs/src/explanation/trust-model.prompt.md
docs/tmp/verbose_extra_context/ha-primary-count-invariant-runner.md
docs/tmp/verbose_extra_context/managed-postgres-roles.md
docs/tmp/verbose_extra_context/process-logging-boundary.md
docs/tmp/verbose_extra_context/trust-model.md


===== docs/tmp/verbose_extra_context/ha-primary-count-invariant-runner.md =====
# HA primary-count invariant runner context

This context file exists only to give K2 exhaustive raw facts about the HA test harness change completed in task `05-task-add-a-perpetual-self-reported-primary-count-invariant-runner.md`.

## Task goal

The task adds a scenario-agnostic invariant runner to the HA acceptance harness. The runner continuously queries each node individually and immediately fails the scenario unless the total number of self-reported primaries is in the allowed set `{0, 1}`.

The important intent is:

- remove split-brain / dual-primary assertions from individual feature text
- remove any older transition-history or ad hoc bookkeeping approach
- make the safety property run for the whole scenario lifetime
- count only what each queried node says about itself
- treat command failure as absence of a self-report for that node
- persist direct structured evidence of the violating sample

## Actual implementation location

The new runner lives in `tests/ha/support/invariant.rs`.

It is wired into the harness lifecycle through `tests/ha/support/world/mod.rs`.

The support module now includes `mod invariant;` in `tests/ha/support/mod.rs`.

## How the runner starts and stops

`HarnessShared::initialize()` now starts the invariant runner before cluster bootstrap continues. That means the invariant is active for the full harness lifetime after the run workspace exists, not only after a later feature step.

The runner is stopped in `HarnessShared::cleanup()` before artifact capture and compose teardown continue.

If startup of the invariant runner fails, `initialize()` performs cleanup and returns an error.

If stopping the invariant runner fails or the runner has already recorded a failure, `cleanup()` records that as a scenario cleanup failure.

## How failures surface to step execution

`HaWorld::harness()` now calls `harness.ensure_background_invariants_healthy()?` before returning the harness reference.

This means normal scenario steps and the existing polling helpers fail through the shared harness access path if the invariant runner has recorded a failure.

The runner also still causes the scenario to fail at cleanup time if the violation happens after the final step but before scenario teardown finishes.

## What is counted as a self-reported primary

The runner does not use the operator-visible authority projection in `state.ha.publication` to count primaries.

Instead it uses the local PostgreSQL role from the node-local `NodeState.pg` value returned by `pgtm status --json` through the host-side observer path:

- `PgInfoState::Primary` counts as one self-reported primary
- `PgInfoState::Replica` counts as not primary
- `PgInfoState::Unknown` counts as not primary
- a command failure for that member counts as absence, not as primary

The runner also validates that the local identity returned by `pgtm status --json` matches the member that was queried. If the queried member returns a different local identity, the runner records that as a runner error because the sample would not be trustworthy.

## Observation path and artifact behavior

The runner uses the existing host-side `PgtmObserver`.

That observer already queries each member individually by materializing a host-side observer config for the specific node, then running `pgtm --config <host-config> --json status`.

The runner transforms the full per-member observation into a smaller sample focused only on per-member self-report:

- member name
- whether it self-reported `primary`
- or whether it self-reported `replica` / `unknown`
- or whether the command failed

On violation, the runner writes a structured artifact:

- `artifacts/primary-count-invariant-violation.json`

That artifact stores:

- `observed_at_ms`
- `allowed_primary_counts`
- `primary_count`
- `members`

Each member entry stores the member name and the reduced self-report result.

The error message surfaced back to the harness mentions the structured artifact path explicitly.

## What was already true before this task

Before this task, the harness already:

- queried node state from the host with `pgtm`
- recorded timeline notes and status snapshots
- used scenario steps and polling helpers to check cluster health / authoritative primary behavior

There was already explanatory doc text in `docs/src/explanation/failure-modes.md` that said the harness samples cluster state from the host and fails if more than one primary is observed.

That existing explanation should now be made more precise:

- the invariant is perpetual / always-on for HA scenarios
- the sample is based on per-node self-reported local PostgreSQL role
- command failure is treated as missing self-report, not as inferred state
- the failure evidence is written to a dedicated structured artifact

## What should not be claimed

Please do not claim any production runtime behavior changed. This is HA test harness behavior, not daemon control-plane logic.

Please do not claim the runner uses cluster-wide inference, authority projection, or timeline-based dual-primary history. The task explicitly removes the need for those approaches.

Please do not claim the runner proves more than it does. It only counts self-reported primaries sampled from each node individually over time.

Please do not claim command failures are treated as a failure by themselves. A command failure alone is treated as absence for that member unless some other runner/internal error occurs.


===== .ralph/tasks/story-ha-acceptance-deletion-first-rewrite/05-task-add-a-perpetual-self-reported-primary-count-invariant-runner.md =====
## Task: Add A Perpetual Self-Reported Primary-Count Invariant Runner <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add a scenario-agnostic invariant runner that continuously queries each node individually and fails immediately unless the sum of self-reported primaries is either 0 or 1. The higher-order goal is to move "no dual primary" logic completely out of feature files and out of ad hoc transition-history bookkeeping. This invariant must run independently of scenario text and must be stronger than the current step-based assertions because it applies for the whole scenario lifetime.

**Context from research and PO direction:**
- The current suite encodes this concern indirectly through feature steps like `there is no dual-primary evidence during the transition window` plus timeline/history helpers such as `assert_no_dual_primary_evidence`.
- The PO explicitly wants this as one of the two stronger continuous expectations:
  - query each node individually
  - look only at how each node reports its own state
  - the total number of primary self-reports must be 0 or 1
  - fail directly when violated
- After task 04, the suite should already have the direct per-node JSON observation surface needed to implement this cleanly.

**Scope:**
- Add the invariant runner under `tests/ha/support/` in whatever minimal module shape best fits the reduced harness.
- Wire the invariant runner into scenario startup/shutdown so it runs automatically for every HA scenario.
- Delete the old dual-primary transition-history bookkeeping once the perpetual runner replaces it.

**Required behavior:**
- Poll every node individually through the direct host-side `pgtm` observation surface.
- Use only each node's self-report to count primaries.
- Treat command failure as absence of a self-report for that node, not as an excuse to infer state from another node.
- Fail immediately on any sample where the primary count is greater than 1 or otherwise outside the allowed `0 | 1` set.
- Emit an artifact or timeline snapshot that makes the failure obvious without log-scraping.

**Expected outcome:**
- The suite has one always-on primary-count safety gate instead of repeated feature-local "no dual primary evidence" steps.
- The world/step layer no longer needs timeline-based dual-primary bookkeeping.
- Scenarios become simpler because this safety property is enforced globally.

</description>

<acceptance_criteria>
- [ ] Add a perpetual HA invariant runner that continuously samples all nodes' self-reported status and enforces the allowed primary-count set `{0, 1}`.
- [ ] Start this runner automatically for every HA scenario and stop/clean it up deterministically at scenario end.
- [ ] Fail the scenario immediately when the invariant is violated; do not wait for a later step to notice.
- [ ] Delete `assert_no_dual_primary_evidence`, transition-window primary-history dependence, and any equivalent feature-local dual-primary assertion machinery after the runner replaces it.
- [ ] Remove all feature text that explicitly asserts "no dual primary evidence"; this property must now live only in the perpetual runner.
- [ ] Persist enough structured failure evidence that the violating sample can be inspected without grepping logs.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Build the perpetual invariant runner on top of the per-node observation surface from task 04.
2. Wire it into the HA scenario lifecycle so it starts automatically and owns its own failure channel/artifacts.
3. Delete the old dual-primary step/history machinery after the runner proves the same property continuously.
4. Run the required repo validation gates after the invariant runner is active in the reduced suite.

### Constraints for execution
- Use only node-local self-reports for this invariant. Do not reintroduce cluster-wide inference or seed-selection heuristics.
- Do not preserve the old timeline/history mechanism as a parallel implementation.
- Keep the failure payload structured and small; the point is direct evidence, not more log scraping.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE


===== tests/ha/support/invariant.rs =====
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use pgtuskmaster_rust::{api::NodeState, pginfo::state::PgInfoState};
use serde::Serialize;

use crate::support::{
    error::{HarnessError, Result},
    observer::pgtm::{ClusterStateObservation, MemberCommandOutcome, PgtmObserver},
    topology::ClusterMember,
};

const VIOLATION_ARTIFACT_NAME: &str = "primary-count-invariant-violation.json";

#[derive(Debug)]
pub struct PrimaryCountInvariantRunner {
    shared: Arc<SharedPrimaryCountInvariantState>,
    join_handle: Option<JoinHandle<Result<()>>>,
}

#[derive(Debug)]
struct SharedPrimaryCountInvariantState {
    stop_requested: AtomicBool,
    failure: Mutex<Option<PrimaryCountInvariantFailure>>,
}

#[derive(Clone, Debug)]
enum PrimaryCountInvariantFailure {
    Violation(PrimaryCountInvariantViolation),
    RunnerError(String),
}

#[derive(Clone, Debug)]
struct PrimaryCountInvariantViolation {
    artifact_path: PathBuf,
    sample: PrimaryCountSample,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct PrimaryCountSample {
    observed_at_ms: u128,
    allowed_primary_counts: [usize; 2],
    primary_count: usize,
    members: Vec<MemberPrimaryCountSample>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MemberPrimaryCountSample {
    member: String,
    self_report: MemberSelfReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum MemberSelfReport {
    Primary,
    NotPrimary {
        pg_state: NonPrimaryPgState,
    },
    CommandFailed {
        message: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum NonPrimaryPgState {
    Replica,
    Unknown,
}

impl PrimaryCountInvariantRunner {
    pub fn start(
        observer: PgtmObserver,
        artifacts_dir: PathBuf,
        poll_interval: Duration,
    ) -> Result<Self> {
        let shared = Arc::new(SharedPrimaryCountInvariantState::new());
        let thread_shared = Arc::clone(&shared);
        let thread_name = "ha-primary-count-invariant".to_string();
        let join_handle = thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                let result =
                    run_primary_count_invariant_loop(observer, artifacts_dir, poll_interval, &thread_shared);
                if let Err(err) = &result {
                    let _ = thread_shared.store_failure(PrimaryCountInvariantFailure::RunnerError(
                        err.to_string(),
                    ));
                }
                result
            })
            .map_err(|err| {
                HarnessError::message(format!(
                    "failed to spawn `{thread_name}` background runner: {err}"
                ))
            })?;

        Ok(Self {
            shared,
            join_handle: Some(join_handle),
        })
    }

    pub fn ensure_healthy(&self) -> Result<()> {
        self.shared.load_failure()?.map_or(Ok(()), |failure| {
            Err(HarnessError::message(failure.message()))
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.shared.stop_requested.store(true, Ordering::SeqCst);
        let joined = self.join_handle.take().map(|handle| {
            handle.join().map_err(|_| {
                HarnessError::message("primary-count invariant runner thread panicked")
            })
        });

        if let Some(result) = joined.transpose()? {
            result?;
        }

        self.ensure_healthy()
    }
}

impl SharedPrimaryCountInvariantState {
    fn new() -> Self {
        Self {
            stop_requested: AtomicBool::new(false),
            failure: Mutex::new(None),
        }
    }

    fn stop_requested(&self) -> bool {
        self.stop_requested.load(Ordering::SeqCst)
    }

    fn load_failure(&self) -> Result<Option<PrimaryCountInvariantFailure>> {
        self.failure
            .lock()
            .map(|failure| failure.clone())
            .map_err(|_| HarnessError::message("primary-count invariant mutex was poisoned"))
    }

    fn store_failure(&self, failure: PrimaryCountInvariantFailure) -> Result<()> {
        self.failure
            .lock()
            .map(|mut slot| {
                if slot.is_none() {
                    *slot = Some(failure);
                }
            })
            .map_err(|_| HarnessError::message("primary-count invariant mutex was poisoned"))
    }
}

impl PrimaryCountInvariantFailure {
    fn message(&self) -> String {
        match self {
            Self::Violation(violation) => format!(
                "primary-count invariant violated: {}. structured sample: {}",
                violation.sample.summary(),
                violation.artifact_path.display()
            ),
            Self::RunnerError(message) => {
                format!("primary-count invariant runner failed: {message}")
            }
        }
    }
}

impl PrimaryCountInvariantViolation {
    fn new(artifact_path: PathBuf, sample: PrimaryCountSample) -> Self {
        Self {
            artifact_path,
            sample,
        }
    }
}

impl PrimaryCountSample {
    fn from_observation(observation: &ClusterStateObservation) -> Result<Self> {
        let members = observation
            .members()
            .iter()
            .map(MemberPrimaryCountSample::from_observation)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            observed_at_ms: timestamp_millis()?,
            allowed_primary_counts: [0, 1],
            primary_count: members
                .iter()
                .filter(|member| member.self_report.is_primary())
                .count(),
            members,
        })
    }

    fn violates_allowed_primary_counts(&self) -> bool {
        !self.allowed_primary_counts.contains(&self.primary_count)
    }

    fn summary(&self) -> String {
        format!(
            "observed {} self-reported primaries ({})",
            self.primary_count,
            self.members
                .iter()
                .map(MemberPrimaryCountSample::summary)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl MemberPrimaryCountSample {
    fn from_observation(
        observation: &crate::support::observer::pgtm::MemberStateObservation,
    ) -> Result<Self> {
        let member = observation.member;
        let self_report = match &observation.outcome {
            MemberCommandOutcome::Observed(output) => classify_self_report(member, &output.state)?,
            MemberCommandOutcome::Failed(message) => MemberSelfReport::CommandFailed {
                message: message.clone(),
            },
        };

        Ok(Self {
            member: member.service_name().to_string(),
            self_report,
        })
    }

    fn summary(&self) -> String {
        format!("{}={}", self.member, self.self_report.summary())
    }
}

impl MemberSelfReport {
    fn is_primary(&self) -> bool {
        matches!(self, Self::Primary)
    }

    fn summary(&self) -> String {
        match self {
            Self::Primary => "primary".to_string(),
            Self::NotPrimary { pg_state } => format!("not_primary({})", pg_state.label()),
            Self::CommandFailed { .. } => "command_failed".to_string(),
        }
    }
}

impl NonPrimaryPgState {
    fn label(&self) -> &'static str {
        match self {
            Self::Replica => "replica",
            Self::Unknown => "unknown",
        }
    }
}

fn run_primary_count_invariant_loop(
    observer: PgtmObserver,
    artifacts_dir: PathBuf,
    poll_interval: Duration,
    shared: &SharedPrimaryCountInvariantState,
) -> Result<()> {
    while !shared.stop_requested() {
        let observation = observer.observe_states()?;
        let sample = PrimaryCountSample::from_observation(&observation)?;
        if sample.violates_allowed_primary_counts() {
            let artifact_path = artifacts_dir.join(VIOLATION_ARTIFACT_NAME);
            persist_violation_sample(artifact_path.as_path(), &sample)?;
            shared.store_failure(PrimaryCountInvariantFailure::Violation(
                PrimaryCountInvariantViolation::new(artifact_path, sample),
            ))?;
            return Ok(());
        }
        thread::sleep(poll_interval);
    }

    Ok(())
}

fn classify_self_report(member: ClusterMember, state: &NodeState) -> Result<MemberSelfReport> {
    let reported_member = state.identity.member_id.as_str();
    if reported_member != member.service_name() {
        return Err(HarnessError::message(format!(
            "pgtm status via `{member}` returned local identity `{reported_member}`"
        )));
    }

    Ok(match state.pg {
        PgInfoState::Primary { .. } => MemberSelfReport::Primary,
        PgInfoState::Replica { .. } => MemberSelfReport::NotPrimary {
            pg_state: NonPrimaryPgState::Replica,
        },
        PgInfoState::Unknown { .. } => MemberSelfReport::NotPrimary {
            pg_state: NonPrimaryPgState::Unknown,
        },
    })
}

fn persist_violation_sample(path: &Path, sample: &PrimaryCountSample) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let rendered = serde_json::to_string_pretty(sample).map_err(|source| HarnessError::Json {
        context: "serializing primary-count invariant violation".to_string(),
        source,
    })?;
    write_text_file(path, rendered.as_str())
}

fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn timestamp_millis() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_count_sample_detects_dual_primary_violation() {
        let sample = PrimaryCountSample {
            observed_at_ms: 1,
            allowed_primary_counts: [0, 1],
            primary_count: 2,
            members: vec![
                MemberPrimaryCountSample {
                    member: "node-a".to_string(),
                    self_report: MemberSelfReport::Primary,
                },
                MemberPrimaryCountSample {
                    member: "node-b".to_string(),
                    self_report: MemberSelfReport::Primary,
                },
                MemberPrimaryCountSample {
                    member: "node-c".to_string(),
                    self_report: MemberSelfReport::NotPrimary {
                        pg_state: NonPrimaryPgState::Replica,
                    },
                },
            ],
        };

        assert!(sample.violates_allowed_primary_counts());
    }
}


===== tests/ha/support/world/mod.rs =====
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cucumber::World;
use pgtuskmaster_rust::{
    api::NodeState,
    ha::types::{AuthorityProjection, PublicationState},
};

use crate::support::{
    docker::{cli::DockerCli, ryuk::RyukGuard},
    error::{HarnessError, Result},
    faults::{
        append_fault_rule_script, clear_fault_rules_script, ensure_fault_plumbing_script,
        remove_fault_rule_script, BlockerKind, TrafficPath, DATABASE_MEMBERS, FAULT_DIR,
    },
    feature_metadata,
    givens::{
        resolve_given, ComposeVariant, FixtureMaterialization, HaGivenDefinition, HaGivenId,
        MemberRuntimeConfigMaterialization, NodeRuntimeTemplate, SharedFixtureEntry,
    },
    invariant::PrimaryCountInvariantRunner,
    observer::{
        pgtm::{MemberCommandOutcome, PgtmObserver},
        sql::SqlObserver,
    },
    timeouts::TimeoutModel,
    topology::{ClusterMember, ComposeService},
};

#[derive(Debug, Default, World)]
pub struct HaWorld {
    pub harness: Option<HarnessShared>,
    pub scenario: ScenarioState,
}

#[derive(Debug, Default)]
pub struct ScenarioState {
    pub name: Option<String>,
    pub aliases: AliasRegistry,
    pub availability: ScenarioAvailability,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AliasName(String);

impl From<&str> for AliasName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for AliasName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemberSet {
    members: BTreeSet<ClusterMember>,
}

impl MemberSet {
    pub fn insert(&mut self, member: ClusterMember) -> bool {
        self.members.insert(member)
    }

    pub fn remove(&mut self, member: ClusterMember) -> bool {
        self.members.remove(&member)
    }

    pub fn contains(&self, member: ClusterMember) -> bool {
        self.members.contains(&member)
    }

    pub fn clear(&mut self) {
        self.members.clear();
    }
}

#[derive(Debug, Default)]
pub struct AliasRegistry {
    pub aliases_by_name: BTreeMap<AliasName, ClusterMember>,
}

#[derive(Debug, Default)]
pub struct ScenarioAvailability {
    pub stopped_members: MemberSet,
    pub observer_unreachable_members: MemberSet,
}

impl HaWorld {
    pub fn reset(&mut self) {
        self.harness = None;
        self.scenario = ScenarioState::default();
    }

    pub fn harness(&self) -> Result<&HarnessShared> {
        let harness = self
            .harness
            .as_ref()
            .ok_or_else(|| HarnessError::message("scenario harness has not been initialized"))?;
        harness.ensure_background_invariants_healthy()?;
        Ok(harness)
    }

    pub fn set_scenario_name(&mut self, scenario_name: String) {
        self.scenario.name = Some(scenario_name);
    }

    pub fn scenario_name(&self) -> Result<&str> {
        self.scenario
            .name
            .as_deref()
            .ok_or_else(|| HarnessError::message("scenario name has not been initialized"))
    }

    pub fn set_harness(&mut self, harness: HarnessShared) {
        self.harness = Some(harness);
    }

    pub fn remember_member_alias(&mut self, alias: impl Into<AliasName>, member: ClusterMember) {
        self.scenario
            .aliases
            .aliases_by_name
            .insert(alias.into(), member);
    }

    pub fn require_member_alias(&self, alias: &str) -> Result<ClusterMember> {
        self.scenario
            .aliases
            .aliases_by_name
            .get(&AliasName::from(alias))
            .copied()
            .ok_or_else(|| HarnessError::message(format!("alias `{alias}` was not recorded")))
    }

    pub fn add_stopped_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.availability.stopped_members.insert(member);
    }

    pub fn remove_stopped_node(&mut self, member: ClusterMember) {
        let _ = self.scenario.availability.stopped_members.remove(member);
    }

    pub fn mark_observer_unreachable(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .availability
            .observer_unreachable_members
            .insert(member);
    }

    pub fn clear_observer_unreachable(&mut self, member: ClusterMember) {
        let _ = self
            .scenario
            .availability
            .observer_unreachable_members
            .remove(member);
    }

    pub fn clear_observer_unreachable_members(&mut self) {
        self.scenario
            .availability
            .observer_unreachable_members
            .clear();
    }

    pub fn cleanup(&mut self) -> Result<()> {
        let cleanup_result = match self.harness.as_mut() {
            Some(harness) => harness.cleanup(),
            None => Ok(()),
        };
        self.reset();

        cleanup_result
    }
}

#[derive(Debug)]
pub struct HarnessWorkspace {
    pub run_id: String,
    pub feature_name: String,
    pub given: HaGivenDefinition,
    pub paths: WorkspacePaths,
}

#[derive(Debug)]
pub struct WorkspacePaths {
    pub run_dir: PathBuf,
    pub materialized_dir: PathBuf,
    pub artifacts_dir: PathBuf,
}

#[derive(Debug)]
pub struct ComposeStack {
    pub file: PathBuf,
    pub project: String,
}

#[derive(Debug)]
pub struct HarnessShared {
    pub workspace: HarnessWorkspace,
    pub compose: ComposeStack,
    pub cucumber_test_image_run_id: String,
    pub docker: DockerCli,
    pub ryuk: Option<RyukGuard>,
    pub timeouts: TimeoutModel,
    service_container_ids: Mutex<BTreeMap<ComposeService, String>>,
    timeline: Mutex<Vec<serde_json::Value>>,
    primary_count_invariant: Option<PrimaryCountInvariantRunner>,
    cleaned_up: bool,
}

impl HarnessShared {
    pub async fn initialize(given: HaGivenId, scenario_name: &str) -> Result<Self> {
        let feature = feature_metadata()?;
        let docker = DockerCli::discover()?;
        docker.verify_daemon()?;

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), given)?;
        let run_id = build_run_id(feature.feature_name.as_str(), scenario_name)?;
        let compose = ComposeStack {
            file: PathBuf::new(),
            project: build_compose_project(feature.feature_name.as_str(), run_id.as_str()),
        };
        let cucumber_test_image_run_id = required_env("PGTM_CUCUMBER_TEST_RUN_ID")?;
        let paths = WorkspacePaths {
            run_dir: repo_root
                .join("tests/ha/runs")
                .join(feature.feature_name.as_str())
                .join(run_id.as_str()),
            materialized_dir: repo_root
                .join("tests/ha/runs")
                .join(feature.feature_name.as_str())
                .join(run_id.as_str())
                .join("materialized"),
            artifacts_dir: repo_root
                .join("tests/ha/runs")
                .join(feature.feature_name.as_str())
                .join(run_id.as_str())
                .join("artifacts"),
        };
        create_dir_all(paths.run_dir.as_path())?;
        create_dir_all(paths.materialized_dir.as_path())?;
        create_dir_all(paths.artifacts_dir.as_path())?;
        materialize_given_fixture(&given, paths.materialized_dir.as_path())?;
        create_fault_directories(paths.materialized_dir.as_path())?;

        let compose = ComposeStack {
            file: paths.materialized_dir.join("compose.yml"),
            project: compose.project,
        };
        let timeouts = TimeoutModel::from_runtime_config(
            paths
                .materialized_dir
                .join(ClusterMember::SEED_PRIMARY.runtime_config_relative_path())
                .as_path(),
        )?;
        let ryuk = RyukGuard::start(docker.clone(), compose.project.as_str())?;
        let dcs_service_names = given
            .dcs_services()
            .into_iter()
            .map(|service| service.service_name())
            .collect::<Vec<_>>();
        docker.compose_up_services(
            compose.file.as_path(),
            compose.project.as_str(),
            dcs_service_names.as_slice(),
        )?;
        let service_container_ids = Mutex::new(BTreeMap::new());

        let mut harness = Self {
            workspace: HarnessWorkspace {
                run_id,
                feature_name: feature.feature_name.clone(),
                given,
                paths,
            },
            compose,
            cucumber_test_image_run_id,
            docker,
            ryuk: Some(ryuk),
            timeouts,
            service_container_ids,
            timeline: Mutex::new(Vec::new()),
            primary_count_invariant: None,
            cleaned_up: false,
        };
        harness.record_note(
            "initialize",
            format!(
                "created per-feature run workspace using cucumber image run id `{}`",
                harness.cucumber_test_image_run_id
            ),
        )?;
        if let Err(err) = harness.start_primary_count_invariant() {
            let cleanup_error = harness.cleanup().err();
            return match cleanup_error {
                None => Err(err),
                Some(cleanup) => Err(HarnessError::message(format!(
                    "{err}\ncleanup after invariant startup failure also failed: {cleanup}"
                ))),
            };
        }
        if let Err(err) = harness.bootstrap_cluster().await {
            let cleanup_error = harness.cleanup().err();
            return match cleanup_error {
                None => Err(err),
                Some(cleanup) => Err(HarnessError::message(format!(
                    "{err}\ncleanup after bootstrap failure also failed: {cleanup}"
                ))),
            };
        }
        Ok(harness)
    }

    pub fn feature_name(&self) -> &str {
        self.workspace.feature_name.as_str()
    }

    pub fn run_id(&self) -> &str {
        self.workspace.run_id.as_str()
    }

    pub fn given_name(&self) -> &str {
        self.workspace.given.id.as_str()
    }

    pub fn compose_file(&self) -> &Path {
        self.compose.file.as_path()
    }

    pub fn compose_project(&self) -> &str {
        self.compose.project.as_str()
    }

    pub fn run_dir(&self) -> &Path {
        self.workspace.paths.run_dir.as_path()
    }

    pub fn materialized_dir(&self) -> &Path {
        self.workspace.paths.materialized_dir.as_path()
    }

    pub fn artifacts_dir(&self) -> &Path {
        self.workspace.paths.artifacts_dir.as_path()
    }

    pub fn observer(&self) -> PgtmObserver {
        PgtmObserver::new(
            self.docker.clone(),
            self.compose.file.clone(),
            self.compose.project.clone(),
            self.materialized_dir().to_path_buf(),
        )
    }

    pub fn ensure_background_invariants_healthy(&self) -> Result<()> {
        self.primary_count_invariant
            .as_ref()
            .map_or(Ok(()), PrimaryCountInvariantRunner::ensure_healthy)
    }

    pub fn sql(&self) -> SqlObserver {
        SqlObserver::new(self.materialized_dir().to_path_buf())
    }

    pub fn kill_node(&self, member: ClusterMember) -> Result<()> {
        let container_id = self.service_container_id(member.into())?;
        self.record_note("docker.kill", format!("killing `{member}`"))?;
        self.docker.kill_container(container_id.as_str())
    }

    pub fn start_node(&self, member: ClusterMember) -> Result<()> {
        let container_id = self.service_container_id(member.into())?;
        self.record_note("docker.start", format!("starting `{member}`"))?;
        self.docker.start_container(container_id.as_str())
    }

    pub fn record_note(&self, phase: &str, detail: impl Into<String>) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "note",
            "phase": phase,
            "detail": detail.into(),
            "timestamp_ms": timestamp_millis()?,
        }))
    }

    pub fn record_status_snapshot(&self, phase: &str, status: &NodeState) -> Result<()> {
        self.push_timeline_entry(serde_json::json!({
            "kind": "status",
            "phase": phase,
            "timestamp_ms": timestamp_millis()?,
            "status": status,
        }))
    }

    pub fn service_container_id(&self, service: ComposeService) -> Result<String> {
        if let Some(container_id) = self.cached_service_container_id(service)? {
            return Ok(container_id);
        }
        self.refresh_service_container_ids()?;
        self.cached_service_container_id(service)?.ok_or_else(|| {
            HarnessError::message(format!(
                "docker compose service `{service}` has no container in project `{}`",
                self.compose_project()
            ))
        })
    }

    pub fn stop_service(&self, service: ComposeService) -> Result<()> {
        let container_id = self.service_container_id(service)?;
        self.record_note("docker.stop_service", format!("stopping `{service}`"))?;
        self.docker.kill_container(container_id.as_str())
    }

    pub fn start_service(&self, service: ComposeService) -> Result<()> {
        let container_id = self.service_container_id(service)?;
        self.record_note("docker.start_service", format!("starting `{service}`"))?;
        self.docker.start_container(container_id.as_str())
    }

    pub fn run_shell_as_root(&self, service: ComposeService, script: &str) -> Result<String> {
        let container_id = self.service_container_id(service)?;
        self.docker.exec_as_user(
            container_id.as_str(),
            "root",
            Path::new("/bin/sh"),
            &["-lc", script],
        )
    }

    pub fn ensure_fault_plumbing(&self, service: ComposeService) -> Result<()> {
        let script = ensure_fault_plumbing_script();
        let _ = self.run_shell_as_root(service, script.as_str())?;
        self.record_note("fault.ensure_plumbing", format!("service={service}"))?;
        Ok(())
    }

    pub fn clear_network_faults(&self, service: ComposeService) -> Result<()> {
        if !self.service_is_running(service)? {
            self.record_note(
                "fault.clear_network",
                format!("service={service} skipped=container_not_running"),
            )?;
            return Ok(());
        }
        let script = clear_fault_rules_script();
        if let Err(err) = self.run_shell_as_root(service, script.as_str()) {
            if container_not_running_error(&err) {
                self.record_note(
                    "fault.clear_network",
                    format!("service={service} skipped=container_not_running"),
                )?;
                return Ok(());
            }
            return Err(err);
        }
        self.record_note("fault.clear_network", format!("service={service}"))?;
        Ok(())
    }

    pub fn heal_member_network_faults(&self, member: ClusterMember) -> Result<()> {
        self.clear_network_faults(member.into())?;
        for peer in DATABASE_MEMBERS {
            if peer == member {
                continue;
            }
            for path in [TrafficPath::Postgres, TrafficPath::Api, TrafficPath::Dcs] {
                self.unblock_member_path_to_host(peer, path, member.into())?;
            }
        }
        self.record_note("fault.heal_member_network", format!("member={member}"))?;
        Ok(())
    }

    pub fn block_member_path_to_host(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_service: ComposeService,
    ) -> Result<()> {
        let peer_container_id = self.service_container_id(peer_service)?;
        let peer_ip = self
            .docker
            .container_ipv4_address(peer_container_id.as_str())?;
        self.block_member_path_to_address(
            member,
            path,
            peer_ip.as_str(),
            peer_service.service_name(),
        )
    }

    pub fn unblock_member_path_to_host(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_service: ComposeService,
    ) -> Result<()> {
        if !self.service_is_running(member.into())? {
            self.record_note(
                "fault.unblock_path",
                format!(
                    "member={member} path={} peer={peer_service} skipped=container_not_running",
                    path.label()
                ),
            )?;
            return Ok(());
        }
        let peer_container_id = self.service_container_id(peer_service)?;
        let peer_ip = self
            .docker
            .container_ipv4_address(peer_container_id.as_str())?;
        let script = remove_fault_rule_script(peer_ip.as_str(), path.port());
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note(
            "fault.unblock_path",
            format!("member={member} path={} peer={peer_service}", path.label()),
        )?;
        Ok(())
    }

    fn block_member_path_to_address(
        &self,
        member: ClusterMember,
        path: TrafficPath,
        peer_ip: &str,
        peer_label: &str,
    ) -> Result<()> {
        self.ensure_fault_plumbing(member.into())?;
        let script = append_fault_rule_script(peer_ip, path.port());
        let _ = self.run_shell_as_root(member.into(), script.as_str())?;
        self.record_note(
            "fault.block_path",
            format!("member={member} path={} peer={peer_label}", path.label()),
        )?;
        Ok(())
    }

    pub fn isolate_member_from_peer_on_path(
        &self,
        member: ClusterMember,
        peer: ClusterMember,
        path: TrafficPath,
    ) -> Result<()> {
        self.block_member_path_to_host(member, path, peer.into())?;
        self.block_member_path_to_host(peer, path, member.into())
    }

    pub fn isolate_member_from_all_peers_on_path(
        &self,
        member: ClusterMember,
        path: TrafficPath,
    ) -> Result<()> {
        DATABASE_MEMBERS
            .into_iter()
            .filter(|peer| *peer != member)
            .try_for_each(|peer| self.isolate_member_from_peer_on_path(member, peer, path))
    }

    pub fn isolate_member_from_observer_on_api(&self, member: ClusterMember) -> Result<()> {
        let gateway_ip = self.member_network_gateway_ipv4(member)?;
        self.block_member_path_to_address(
            member,
            TrafficPath::Api,
            gateway_ip.as_str(),
            "host-operator",
        )
    }

    pub fn cut_member_off_from_dcs(&self, member: ClusterMember) -> Result<()> {
        self.block_member_path_to_host(
            member,
            TrafficPath::Dcs,
            self.workspace.given.local_dcs_service_for(member).into(),
        )
    }

    pub fn stop_all_dcs_services(&self) -> Result<()> {
        self.workspace
            .given
            .dcs_services()
            .into_iter()
            .try_for_each(|service| self.stop_service(service.into()))
    }

    pub fn start_all_dcs_services(&self) -> Result<()> {
        self.workspace
            .given
            .dcs_services()
            .into_iter()
            .try_for_each(|service| self.start_service(service.into()))
    }

    pub fn stop_dcs_quorum_majority(&self) -> Result<()> {
        self.workspace
            .given
            .quorum_majority_dcs_services()
            .into_iter()
            .try_for_each(|service| self.stop_service(service.into()))
    }

    pub fn start_dcs_quorum_majority(&self) -> Result<()> {
        self.workspace
            .given
            .quorum_majority_dcs_services()
            .into_iter()
            .try_for_each(|service| self.start_service(service.into()))
    }

    pub fn stop_member_local_dcs(&self, member: ClusterMember) -> Result<()> {
        self.stop_service(self.workspace.given.local_dcs_service_for(member).into())
    }

    pub fn start_member_local_dcs(&self, member: ClusterMember) -> Result<()> {
        self.start_service(self.workspace.given.local_dcs_service_for(member).into())
    }

    pub fn set_blocker(
        &self,
        member: ClusterMember,
        blocker: BlockerKind,
        enabled: bool,
    ) -> Result<()> {
        if enabled {
            self.write_fault_marker(member, blocker.marker_path())?;
            self.remove_fault_marker(member, blocker.clear_on_start_marker_path())?;
        } else {
            self.remove_fault_marker(member, blocker.marker_path())?;
            self.remove_fault_marker(member, blocker.clear_on_start_marker_path())?;
        }
        self.record_note(
            "fault.blocker",
            format!(
                "member={member} blocker={} enabled={enabled}",
                blocker.label()
            ),
        )?;
        Ok(())
    }

    pub fn wipe_member_data_dir(&self, member: ClusterMember) -> Result<()> {
        let marker_path = "/var/lib/pgtuskmaster/faults/wipe-data-on-start";
        self.write_fault_marker(member, marker_path)?;
        self.record_note("fault.wipe_data_dir", format!("member={member}"))?;
        Ok(())
    }

    pub fn clear_all_network_faults(&self) -> Result<()> {
        for service in DATABASE_MEMBERS.into_iter().map(ComposeService::from) {
            self.clear_network_faults(service)?;
        }
        Ok(())
    }

    fn service_is_running(&self, service: ComposeService) -> Result<bool> {
        let container_id = self.service_container_id(service)?;
        Ok(self.docker.container_state_status(container_id.as_str())? == "running")
    }

    fn host_fault_dir(&self, member: ClusterMember) -> PathBuf {
        self.materialized_dir()
            .join("faults")
            .join(member.service_name())
    }

    fn member_network_gateway_ipv4(&self, member: ClusterMember) -> Result<String> {
        let container_id = self.service_container_id(member.into())?;
        self.docker.container_network_gateway(container_id.as_str())
    }

    fn host_fault_marker_path(&self, member: ClusterMember, marker_path: &str) -> Result<PathBuf> {
        let relative_path = Path::new(marker_path)
            .strip_prefix(FAULT_DIR)
            .map_err(|_| {
                HarnessError::message(format!(
                    "fault marker `{marker_path}` does not live under `{FAULT_DIR}`"
                ))
            })?;
        Ok(self.host_fault_dir(member).join(relative_path))
    }

    fn write_fault_marker(&self, member: ClusterMember, marker_path: &str) -> Result<()> {
        let marker_file = self.host_fault_marker_path(member, marker_path)?;
        if let Some(parent) = marker_file.parent() {
            create_dir_all(parent)?;
        }
        write_text_file(marker_file.as_path(), "")?;
        Ok(())
    }

    fn remove_fault_marker(&self, member: ClusterMember, marker_path: &str) -> Result<()> {
        let marker_file = self.host_fault_marker_path(member, marker_path)?;
        match fs::remove_file(marker_file.as_path()) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(HarnessError::Io {
                path: marker_file,
                source,
            }),
        }
    }

    async fn bootstrap_cluster(&self) -> Result<()> {
        for service in self.workspace.given.dcs_services() {
            self.wait_for_service_health(service.into()).await?;
        }
        self.record_note("bootstrap", "starting seed primary node-b")?;
        self.docker.compose_up_services(
            self.compose_file(),
            self.compose_project(),
            &["node-b"],
        )?;
        self.wait_for_seed_primary().await?;
        self.record_note("bootstrap", "starting remaining nodes node-a and node-c")?;
        self.docker.compose_up_services(
            self.compose_file(),
            self.compose_project(),
            &["node-a", "node-c"],
        )
    }

    async fn wait_for_service_health(&self, service: ComposeService) -> Result<()> {
        let deadline = Instant::now() + self.timeouts.startup_deadline;
        let mut last_error = None;
        while Instant::now() < deadline {
            let result = match self
                .service_container_id(service)
                .and_then(|container_id| self.docker.container_health_status(container_id.as_str()))
            {
                Ok(Some(status)) if status == "healthy" => Ok(()),
                Ok(Some(status)) => Err(HarnessError::message(format!(
                    "service `{service}` health is `{status}`"
                ))),
                Ok(None) => Err(HarnessError::message(format!(
                    "service `{service}` does not expose a docker health status"
                ))),
                Err(err) => Err(err),
            };
            match result {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            tokio::time::sleep(self.timeouts.poll_interval).await;
        }

        Err(HarnessError::message(format!(
            "timed out waiting for service `{service}` to become healthy; last observed error: {}",
            last_error.unwrap_or_else(|| "no health state was observed".to_string())
        )))
    }

    async fn wait_for_seed_primary(&self) -> Result<()> {
        let deadline = Instant::now() + self.timeouts.startup_deadline;
        let mut last_error = None;
        while Instant::now() < deadline {
            let result = match self
                .observer()
                .state_via_member(ClusterMember::SEED_PRIMARY)
            {
                Ok(status) => {
                    self.record_status_snapshot("bootstrap.seed_primary", &status)?;
                    validate_seed_primary(&status)
                }
                Err(err) => Err(err),
            };
            match result {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err.to_string()),
            }
            tokio::time::sleep(self.timeouts.poll_interval).await;
        }

        Err(HarnessError::message(format!(
            "timed out waiting for bootstrap primary before starting replicas; last observed error: {}",
            last_error.unwrap_or_else(|| "no status was observed".to_string())
        )))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        if self.cleaned_up {
            return Ok(());
        }

        let mut failures = Vec::new();
        let compose_network = format!("{}_ha", self.compose_project());
        if let Some(runner) = self.primary_count_invariant.as_mut() {
            if let Err(err) = runner.stop() {
                failures.push(format!("primary-count invariant cleanup failed: {err}"));
            }
        }
        let capture_result = self.capture_artifacts();
        if let Err(err) = &capture_result {
            failures.push(format!("artifact capture failed: {err}"));
        }
        let compose_result = self
            .docker
            .compose_down(self.compose_file(), self.compose_project());
        if let Err(err) = &compose_result {
            failures.push(format!("docker compose down failed: {err}"));
        }
        let network_result = if compose_result.is_ok() {
            self.docker
                .wait_for_network_absent(compose_network.as_str())
        } else {
            Ok(())
        };
        if let Err(err) = &network_result {
            failures.push(format!("compose network cleanup failed: {err}"));
        }
        let ryuk_result = self.ryuk.as_mut().map(RyukGuard::close).transpose();
        if let Err(err) = &ryuk_result {
            failures.push(format!("ryuk cleanup failed: {err}"));
        }
        if compose_result.is_ok() && network_result.is_ok() && ryuk_result.is_ok() {
            self.cleaned_up = true;
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(HarnessError::message(failures.join("\n")))
        }
    }

    fn capture_artifacts(&self) -> Result<()> {
        let mut failures = Vec::new();
        write_text_file(
            self.artifacts_dir().join("compose-ps.json").as_path(),
            serde_json::to_string_pretty(
                &self
                    .docker
                    .compose_ps_entries(self.compose_file(), self.compose_project())?,
            )
            .map_err(|source| HarnessError::Json {
                context: "serializing docker compose ps json".to_string(),
                source,
            })?
            .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir().join("compose-logs.txt").as_path(),
            self.docker
                .compose_logs(self.compose_file(), self.compose_project())?
                .as_str(),
        )?;
        write_text_file(
            self.artifacts_dir().join("run-metadata.json").as_path(),
            serde_json::to_string_pretty(&serde_json::json!({
                "feature_name": self.feature_name(),
                "given_name": self.given_name(),
                "run_id": self.run_id(),
                "run_dir": self.run_dir(),
                "materialized_dir": self.materialized_dir(),
                "artifacts_dir": self.artifacts_dir(),
                "compose_project": self.compose_project(),
                "cucumber_test_image_run_id": self.cucumber_test_image_run_id,
            }))
            .map_err(|source| HarnessError::Json {
                context: "serializing run metadata".to_string(),
                source,
            })?
            .as_str(),
        )?;
        let timeline = self.timeline_entries()?;
        write_text_file(
            self.artifacts_dir().join("timeline.json").as_path(),
            serde_json::to_string_pretty(&timeline)
                .map_err(|source| HarnessError::Json {
                    context: "serializing cucumber timeline".to_string(),
                    source,
                })?
                .as_str(),
        )?;
        match self.observer().observe_states() {
            Ok(states) => {
                let serialized = states
                    .members()
                    .iter()
                    .map(|observation| match &observation.outcome {
                        MemberCommandOutcome::Observed(output) => serde_json::json!({
                            "member": observation.member.service_name(),
                            "state": output,
                        }),
                        MemberCommandOutcome::Failed(message) => serde_json::json!({
                            "member": observation.member.service_name(),
                            "failure": message,
                        }),
                    })
                    .collect::<Vec<_>>();
                write_text_file(
                    self.artifacts_dir().join("operator-state.json").as_path(),
                    serde_json::to_string_pretty(&serialized)
                        .map_err(|source| HarnessError::Json {
                            context: "serializing operator state payload".to_string(),
                            source,
                        })?
                        .as_str(),
                )?
            }
            Err(err) => failures.push(format!("operator state capture failed: {err}")),
        }

        for service in self.workspace.given.artifact_services() {
            match self.service_container_id(service) {
                Ok(container_id) => match self.docker.inspect_container(container_id.as_str()) {
                    Ok(inspect) => {
                        let artifact = self
                            .artifacts_dir()
                            .join(format!("inspect-{}.json", service.service_name()));
                        write_text_file(artifact.as_path(), inspect.as_str())?;
                    }
                    Err(err) => failures.push(format!(
                        "docker inspect artifact capture failed for `{service}`: {err}"
                    )),
                },
                Err(err) => failures.push(format!(
                    "container resolution failed for artifact capture `{service}`: {err}"
                )),
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(HarnessError::message(failures.join("\n")))
        }
    }

    fn push_timeline_entry(&self, entry: serde_json::Value) -> Result<()> {
        let mut timeline = self
            .timeline
            .lock()
            .map_err(|_| HarnessError::message("timeline mutex was poisoned"))?;
        timeline.push(entry);
        Ok(())
    }

    fn timeline_entries(&self) -> Result<Vec<serde_json::Value>> {
        self.timeline
            .lock()
            .map(|guard| guard.clone())
            .map_err(|_| HarnessError::message("timeline mutex was poisoned"))
    }

    fn cached_service_container_id(&self, service: ComposeService) -> Result<Option<String>> {
        self.service_container_ids
            .lock()
            .map(|cache| cache.get(&service).cloned())
            .map_err(|_| HarnessError::message("service container cache mutex was poisoned"))
    }

    fn refresh_service_container_ids(&self) -> Result<()> {
        let compose_entries = self
            .docker
            .compose_ps_entries(self.compose_file(), self.compose_project())?;
        let mut cache = self
            .service_container_ids
            .lock()
            .map_err(|_| HarnessError::message("service container cache mutex was poisoned"))?;
        compose_entries
            .into_iter()
            .filter_map(|entry| {
                self.compose_service_for_name(entry.service.as_str())
                    .map(|service| (service, entry.id))
            })
            .for_each(|(service, container_id)| {
                let _ = cache.insert(service, container_id);
            });
        Ok(())
    }

    fn compose_service_for_name(&self, service_name: &str) -> Option<ComposeService> {
        self.workspace
            .given
            .artifact_services()
            .into_iter()
            .find(|service| service.service_name() == service_name)
    }

    fn start_primary_count_invariant(&mut self) -> Result<()> {
        self.primary_count_invariant = Some(PrimaryCountInvariantRunner::start(
            self.observer(),
            self.artifacts_dir().to_path_buf(),
            self.timeouts.poll_interval,
        )?);
        self.record_note(
            "invariant.primary_count.start",
            "started perpetual self-reported primary-count runner",
        )
    }
}

fn container_not_running_error(err: &HarnessError) -> bool {
    matches!(err, HarnessError::CommandFailed { stderr, .. } if stderr.contains("is not running"))
}

fn build_run_id(feature_name: &str, scenario_name: &str) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))?;
    Ok(format!(
        "{}-{}-{}-{}",
        sanitize(feature_name),
        sanitize(scenario_name),
        timestamp.as_millis(),
        std::process::id()
    ))
}

fn timestamp_millis() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))
}

fn required_env(key: &str) -> Result<String> {
    std::env::var(key).map_err(|err| {
        HarnessError::message(format!(
            "required environment variable `{key}` is missing: {err}"
        ))
    })
}

fn build_compose_project(feature_name: &str, run_id: &str) -> String {
    let feature = sanitize(feature_name);
    let run = sanitize(run_id);
    format!("ha-{}-{}", feature, run)
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
}

fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn copy_file(from: &Path, to: &Path) -> Result<()> {
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|source| HarnessError::Io {
            path: to.to_path_buf(),
            source,
        })?;
    apply_private_key_permissions(to)
}

fn materialize_given_fixture(given: &HaGivenDefinition, materialized_root: &Path) -> Result<()> {
    let FixtureMaterialization {
        shared_root,
        compose_variant,
        copies,
        runtime_configs,
    } = &given.materialization;
    for entry in copies {
        copy_shared_fixture_entry(shared_root.as_path(), materialized_root, entry)?;
    }
    for runtime_config in runtime_configs {
        materialize_runtime_config(materialized_root, runtime_config)?;
    }
    materialize_compose_include_file(materialized_root, *compose_variant)?;
    Ok(())
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn apply_private_key_permissions(path: &Path) -> Result<()> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("key") {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions).map_err(|source| HarnessError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }

    Ok(())
}

fn copy_shared_fixture_entry(
    shared_root: &Path,
    materialized_root: &Path,
    entry: &SharedFixtureEntry,
) -> Result<()> {
    match entry {
        SharedFixtureEntry::Directory {
            source_relative_path,
            target_relative_path,
        } => copy_directory(
            shared_root.join(source_relative_path).as_path(),
            materialized_root.join(target_relative_path).as_path(),
        ),
        SharedFixtureEntry::File {
            source_relative_path,
            target_relative_path,
        } => {
            let target_path = materialized_root.join(target_relative_path);
            if let Some(parent) = target_path.parent() {
                create_dir_all(parent)?;
            }
            copy_file(
                shared_root.join(source_relative_path).as_path(),
                target_path.as_path(),
            )
        }
    }
}

fn copy_directory(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Err(HarnessError::message(format!(
            "source directory does not exist: {}",
            from.display()
        )));
    }

    let mut directories = vec![(from.to_path_buf(), to.to_path_buf())];
    while let Some((current_from, current_to)) = directories.pop() {
        create_dir_all(current_to.as_path())?;
        for entry in fs::read_dir(current_from.as_path()).map_err(|source| HarnessError::Io {
            path: current_from.clone(),
            source,
        })? {
            let entry = entry.map_err(|source| HarnessError::Io {
                path: current_from.clone(),
                source,
            })?;
            let source_path = entry.path();
            let destination_path = current_to.join(entry.file_name());
            if source_path.is_dir() {
                directories.push((source_path, destination_path));
            } else {
                copy_file(source_path.as_path(), destination_path.as_path())?;
            }
        }
    }
    Ok(())
}

fn materialize_runtime_config(
    materialized_root: &Path,
    runtime_config: &MemberRuntimeConfigMaterialization,
) -> Result<()> {
    let target_path = materialized_root.join(runtime_config.member.runtime_config_relative_path());
    if let Some(parent) = target_path.parent() {
        create_dir_all(parent)?;
    }
    let rendered = render_member_runtime_template(&runtime_config.template);
    write_text_file(target_path.as_path(), rendered.as_str())
}

fn materialize_compose_include_file(
    materialized_root: &Path,
    compose_variant: ComposeVariant,
) -> Result<()> {
    let compose_variant_path = compose_variant_absolute_path(compose_variant)?;
    let rendered = format!(
        "include:\n  - path: {}\n    project_directory: {}\n",
        toml_path_string(compose_variant_path.as_path()),
        toml_path_string(materialized_root),
    );
    write_text_file(
        materialized_root.join("compose.yml").as_path(),
        rendered.as_str(),
    )
}

fn compose_variant_absolute_path(compose_variant: ComposeVariant) -> Result<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let absolute = repo_root
        .join("tests/ha/givens")
        .join(compose_variant.relative_path());
    if absolute.is_file() {
        Ok(absolute)
    } else {
        Err(HarnessError::message(format!(
            "static compose variant is missing: {}",
            absolute.display()
        )))
    }
}

fn toml_path_string(path: &Path) -> String {
    format!("{:?}", path.display().to_string())
}

fn render_member_runtime_template(template: &NodeRuntimeTemplate) -> String {
    let member = template.binding.member.service_name();
    let dcs_endpoint = template.binding.dcs_service.client_url();
    let replicator = template.postgres_roles.replicator.as_str();
    let rewinder = template.postgres_roles.rewinder.as_str();
    format!(
        r#"[cluster]
name = "ha-cucumber-cluster"
scope = "ha-cucumber-cluster"
member_id = "{member}"

[postgres.paths]
data_dir = "/var/lib/postgresql/data"
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"

[postgres.network]
listen_host = "{member}"
listen_port = 5432

[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[postgres]
tls = {{ mode = "enabled", identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/{member}.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/{member}.key" }} }}, client_auth = {{ client_ca = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}, client_certificate = "optional" }} }}

[postgres.access]
hba = {{ path = "/etc/pgtuskmaster/pg_hba.conf" }}
ident = {{ path = "/etc/pgtuskmaster/pg_ident.conf" }}

[postgres.extra_gucs]
wal_keep_size = "128MB"

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ type = "file", path = "/run/secrets/postgres-superuser-password" }} }}

[postgres.roles.mandatory.replicator]
username = "{replicator}"
auth = {{ type = "password", password = {{ type = "file", path = "/run/secrets/replicator-password" }} }}

[postgres.roles.mandatory.rewinder]
username = "{rewinder}"
auth = {{ type = "password", password = {{ type = "file", path = "/run/secrets/rewinder-password" }} }}

[dcs]
endpoints = ["{dcs_endpoint}"]

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process.timeouts]
pg_rewind_ms = 120000
bootstrap_ms = 300000
fencing_ms = 30000

[process.binaries.overrides]
postgres = "/usr/local/lib/pgtuskmaster/wrappers/postgres"
pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
pg_rewind = "/usr/local/lib/pgtuskmaster/wrappers/pg_rewind"
initdb = "/usr/lib/postgresql/16/bin/initdb"
pg_basebackup = "/usr/local/lib/pgtuskmaster/wrappers/pg_basebackup"
psql = "/usr/lib/postgresql/16/bin/psql"

[logging]
level = "info"
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = {{ enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }}

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = true
path = "/var/log/pgtuskmaster/runtime.jsonl"
mode = "append"

[api]
listen_addr = "0.0.0.0:8443"
transport = {{ transport = "https", tls = {{ identity = {{ cert_chain = {{ path = "/etc/pgtuskmaster/tls/{member}.crt" }}, private_key = {{ path = "/etc/pgtuskmaster/tls/{member}.key" }} }} }} }}
auth = {{ type = "role_tokens", tokens = {{ read_token = {{ type = "file", path = "/run/secrets/api-read-token" }}, admin_token = {{ type = "file", path = "/run/secrets/api-admin-token" }} }} }}

[pgtm.api]
base_url = "https://{member}:8443"
auth = {{ type = "role_tokens", tokens = {{ read_token = {{ type = "file", path = "/run/secrets/api-read-token" }}, admin_token = {{ type = "file", path = "/run/secrets/api-admin-token" }} }} }}
tls = {{ ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }} }}

[pgtm.postgres.tls]
ca_cert = {{ path = "/etc/pgtuskmaster/tls/ca.crt" }}

[debug]
enabled = true
"#
    )
}

fn create_fault_directories(root: &Path) -> Result<()> {
    let faults_root = root.join("faults");
    create_dir_all(faults_root.as_path())?;
    for member in ClusterMember::ALL {
        create_dir_all(faults_root.join(member.service_name()).as_path())?;
    }
    Ok(())
}

fn validate_seed_primary(status: &NodeState) -> Result<()> {
    let discovered_member_count = status.dcs.member_count();
    if discovered_member_count != 1 {
        return Err(HarnessError::message(format!(
            "expected exactly one discovered member during bootstrap, observed {}; warnings={}",
            discovered_member_count,
            format_bootstrap_warnings(status),
        )));
    }

    match operator_visible_primary(status).as_deref() {
        Some("node-b") => Ok(()),
        Some(primary) => Err(HarnessError::message(format!(
            "expected node-b to bootstrap as the seed primary, observed `{primary}`"
        ))),
        None => Err(HarnessError::message(format!(
            "bootstrap state has no authoritative primary; warnings={}",
            format_bootstrap_warnings(status),
        ))),
    }
}

fn format_bootstrap_warnings(status: &NodeState) -> String {
    let mut warnings = Vec::new();
    if operator_visible_primary(status).is_none() {
        warnings.push("no_primary".to_string());
    }
    if status.dcs.member_count() == 0 {
        warnings.push("no_members".to_string());
    }
    if warnings.is_empty() {
        "none".to_string()
    } else {
        warnings.join(" | ")
    }
}

fn operator_visible_primary(status: &NodeState) -> Option<String> {
    match &status.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Some(epoch.holder.0.clone())
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(name: &str) -> Result<PathBuf> {
        let root = std::env::temp_dir().join(format!(
            "pgtm-ha-world-{name}-{}-{}",
            std::process::id(),
            timestamp_millis()?
        ));
        match fs::remove_dir_all(root.as_path()) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(HarnessError::Io { path: root, source });
            }
        }
        create_dir_all(root.as_path())?;
        Ok(root)
    }

    fn cleanup_directory(path: &Path) -> Result<()> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(HarnessError::Io {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    #[test]
    fn materializes_plain_fixture_from_shared_assets_and_static_include_compose() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), HaGivenId::Plain)?;
        let output_root = temporary_directory("plain")?;

        let result = (|| -> Result<()> {
            materialize_given_fixture(&given, output_root.as_path())?;

            let compose =
                fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                    HarnessError::Io {
                        path: output_root.join("compose.yml"),
                        source,
                    }
                })?;
            let expected_variant = compose_variant_absolute_path(ComposeVariant::SharedSingleDcs)?;
            assert!(compose.contains("include:"));
            assert!(compose.contains(expected_variant.display().to_string().as_str()));
            assert!(compose.contains(output_root.display().to_string().as_str()));

            let runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
                source,
            })?;
            toml::from_str::<pgtuskmaster_rust::config::RuntimeConfig>(runtime.as_str()).map_err(
                |source| {
                    HarnessError::message(format!(
                        "materialized node runtime config failed to parse: {source}"
                    ))
                },
            )?;
            assert!(runtime.contains(r#"username = "replicator""#));
            assert!(runtime.contains(r#"username = "rewinder""#));
            assert!(output_root.join("configs/tls/ca.crt").is_file());
            assert!(output_root.join("secrets/replicator-password").is_file());
            Ok(())
        })();

        let cleanup_result = cleanup_directory(output_root.as_path());
        match (result, cleanup_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
                "{err}\ncleanup also failed: {cleanup}"
            ))),
        }
    }

    #[test]
    fn materializes_custom_roles_without_custom_compose_variant() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), HaGivenId::CustomRoles)?;
        let output_root = temporary_directory("custom-roles")?;

        let result = (|| -> Result<()> {
            materialize_given_fixture(&given, output_root.as_path())?;

            let compose =
                fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                    HarnessError::Io {
                        path: output_root.join("compose.yml"),
                        source,
                    }
                })?;
            let expected_variant = compose_variant_absolute_path(ComposeVariant::SharedSingleDcs)?;
            assert!(compose.contains(expected_variant.display().to_string().as_str()));

            let runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
                source,
            })?;
            assert!(runtime.contains(r#"username = "mirrorbot""#));
            assert!(runtime.contains(r#"username = "rewindbot""#));
            Ok(())
        })();

        let cleanup_result = cleanup_directory(output_root.as_path());
        match (result, cleanup_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
                "{err}\ncleanup also failed: {cleanup}"
            ))),
        }
    }

    #[test]
    fn materializes_three_etcd_fixture_with_node_local_dcs_bindings() -> Result<()> {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let given = resolve_given(repo_root.as_path(), HaGivenId::ThreeEtcd)?;
        let output_root = temporary_directory("three-etcd")?;

        let result = (|| -> Result<()> {
            materialize_given_fixture(&given, output_root.as_path())?;

            let compose =
                fs::read_to_string(output_root.join("compose.yml")).map_err(|source| {
                    HarnessError::Io {
                        path: output_root.join("compose.yml"),
                        source,
                    }
                })?;
            let expected_variant =
                compose_variant_absolute_path(ComposeVariant::ColocatedThreeMemberDcs)?;
            assert!(compose.contains(expected_variant.display().to_string().as_str()));

            let node_a_runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeA.runtime_config_relative_path()),
                source,
            })?;
            toml::from_str::<pgtuskmaster_rust::config::RuntimeConfig>(node_a_runtime.as_str())
                .map_err(|source| {
                    HarnessError::message(format!(
                        "materialized node-a runtime config failed to parse: {source}"
                    ))
                })?;
            assert!(node_a_runtime.contains(r#"endpoints = ["http://etcd-a:2379"]"#));

            let node_b_runtime = fs::read_to_string(
                output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
            )
            .map_err(|source| HarnessError::Io {
                path: output_root.join(ClusterMember::NodeB.runtime_config_relative_path()),
                source,
            })?;
            toml::from_str::<pgtuskmaster_rust::config::RuntimeConfig>(node_b_runtime.as_str())
                .map_err(|source| {
                    HarnessError::message(format!(
                        "materialized node-b runtime config failed to parse: {source}"
                    ))
                })?;
            assert!(node_b_runtime.contains(r#"endpoints = ["http://etcd-b:2379"]"#));
            Ok(())
        })();

        let cleanup_result = cleanup_directory(output_root.as_path());
        match (result, cleanup_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(cleanup)) => Err(cleanup),
            (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
                "{err}\ncleanup also failed: {cleanup}"
            ))),
        }
    }

    #[test]
    fn member_alias_round_trips() -> Result<()> {
        let mut world = HaWorld::default();
        world.remember_member_alias("replica", ClusterMember::NodeB);

        assert_eq!(world.require_member_alias("replica")?, ClusterMember::NodeB);
        Ok(())
    }

    #[test]
    fn missing_alias_reports_error() -> Result<()> {
        let world = HaWorld::default();
        match world.require_member_alias("missing") {
            Ok(member) => Err(HarnessError::message(format!(
                "unexpectedly resolved missing alias to `{member}`"
            ))),
            Err(err) => {
                assert!(err.to_string().contains("missing"));
                Ok(())
            }
        }
    }
}


===== tests/ha/support/observer/pgtm.rs =====
use std::{
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use pgtuskmaster_rust::{
    api::{AcceptedResponse, NodeState},
    command::{CommandOutputDto, StateCommandOutputDto},
};

use crate::support::{
    config::{configured_executable, harness_settings},
    docker::cli::DockerCli,
    error::{HarnessError, Result},
    process::{self, CommandSpec},
    topology::ClusterMember,
};

pub type ClusterStatusView = NodeState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryRoutingTarget {
    pub member: ClusterMember,
    pub dsn: String,
}

#[derive(Clone, Debug)]
pub enum MemberCommandOutcome<T> {
    Observed(T),
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct MemberStateObservation {
    pub member: ClusterMember,
    pub outcome: MemberCommandOutcome<StateCommandOutputDto>,
}

#[derive(Clone, Debug)]
pub struct ClusterStateObservation {
    members: Vec<MemberStateObservation>,
}

impl MemberStateObservation {
    pub fn output(&self) -> Option<&StateCommandOutputDto> {
        match &self.outcome {
            MemberCommandOutcome::Observed(output) => Some(output),
            MemberCommandOutcome::Failed(_) => None,
        }
    }

    pub fn state(&self) -> Option<&NodeState> {
        self.output().map(|output| &output.state)
    }

    pub fn failure(&self) -> Option<&str> {
        match &self.outcome {
            MemberCommandOutcome::Observed(_) => None,
            MemberCommandOutcome::Failed(message) => Some(message.as_str()),
        }
    }
}

impl ClusterStateObservation {
    pub fn members(&self) -> &[MemberStateObservation] {
        self.members.as_slice()
    }

    pub fn member(&self, member: ClusterMember) -> Result<&MemberStateObservation> {
        self.members
            .iter()
            .find(|observation| observation.member == member)
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "cluster observation did not include member `{member}`"
                ))
            })
    }
}

#[derive(Clone, Debug)]
pub struct PgtmObserver {
    docker: DockerCli,
    compose_file: PathBuf,
    compose_project: String,
    materialized_dir: PathBuf,
}

impl PgtmObserver {
    pub fn new(
        docker: DockerCli,
        compose_file: PathBuf,
        compose_project: String,
        materialized_dir: PathBuf,
    ) -> Self {
        Self {
            docker,
            compose_file,
            compose_project,
            materialized_dir,
        }
    }

    pub fn observe_states(&self) -> Result<ClusterStateObservation> {
        let members = ClusterMember::ALL
            .into_iter()
            .map(|member| self.observe_state_via_member(member))
            .collect::<Result<Vec<_>>>()?;
        Ok(ClusterStateObservation { members })
    }

    pub fn state_via_member(&self, member: ClusterMember) -> Result<ClusterStatusView> {
        let observation = self.observe_state_via_member(member)?;
        match observation.outcome {
            MemberCommandOutcome::Observed(output) => Ok(output.state),
            MemberCommandOutcome::Failed(message) => Err(HarnessError::message(format!(
                "pgtm status via `{member}` failed: {message}"
            ))),
        }
    }

    pub fn primary_routing_target(
        &self,
        primary_member: ClusterMember,
    ) -> Result<PrimaryRoutingTarget> {
        let published_port = self.member_published_port(primary_member, "5432/tcp")?;
        Ok(PrimaryRoutingTarget {
            member: primary_member,
            dsn: host_postgres_dsn(
                primary_member,
                published_port,
                self.ca_cert_path().as_path(),
                self.observer_cert_path().as_path(),
                self.observer_key_path().as_path(),
            ),
        })
    }

    pub fn switchover_request_via_member(
        &self,
        member: ClusterMember,
        target: Option<ClusterMember>,
    ) -> Result<String> {
        let runtime_config = self.materialize_host_observer_config(member)?;
        let request_args = target
            .into_iter()
            .flat_map(|target_member| {
                [
                    "--switchover-to".to_string(),
                    target_member.service_name().to_string(),
                ]
            })
            .collect::<Vec<_>>();
        let output = self.run_command_via_member(
            member,
            runtime_config.as_path(),
            ["switchover".to_string(), "request".to_string()]
                .into_iter()
                .chain(request_args)
                .collect::<Vec<_>>(),
            "pgtm switchover request",
            extract_switchover_output,
        )?;
        match output {
            MemberCommandOutcome::Observed(accepted) => {
                serde_json::to_string(&accepted).map_err(|source| {
                    HarnessError::message(format!(
                        "serializing switchover response failed: {source}"
                    ))
                })
            }
            MemberCommandOutcome::Failed(message) => Err(HarnessError::message(format!(
                "pgtm switchover request via `{member}` failed: {message}"
            ))),
        }
    }

    fn observe_state_via_member(&self, member: ClusterMember) -> Result<MemberStateObservation> {
        let outcome = match self.materialize_host_observer_config(member) {
            Ok(runtime_config) => self.run_command_via_member(
                member,
                runtime_config.as_path(),
                vec!["status".to_string()],
                "pgtm status",
                extract_state_command_output,
            )?,
            Err(err) => MemberCommandOutcome::Failed(err.to_string()),
        };
        Ok(MemberStateObservation { member, outcome })
    }

    fn run_command_via_member<T>(
        &self,
        member: ClusterMember,
        runtime_config: &Path,
        command_args: Vec<String>,
        context_label: &str,
        decode_output: fn(CommandOutputDto) -> Result<T>,
    ) -> Result<MemberCommandOutcome<T>> {
        let binary = resolve_pgtm_binary()?;
        let args = [
            "--config".to_string(),
            runtime_config.display().to_string(),
            "--json".to_string(),
        ]
        .into_iter()
        .chain(command_args)
        .collect::<Vec<_>>();
        let context = format!("{context_label} via `{member}`");
        let output = process::run(
            CommandSpec::new(binary.clone(), context.clone())
                .env("PATH", "")
                .args(args.as_slice()),
        );
        match output {
            Ok(stdout) => {
                let rendered = stdout.stdout_text(format!("{context} stdout"))?;
                let dto = serde_json::from_str::<CommandOutputDto>(rendered.as_str()).map_err(
                    |source| HarnessError::Json {
                        context: context.clone(),
                        source,
                    },
                )?;
                decode_output(dto).map(MemberCommandOutcome::Observed)
            }
            Err(HarnessError::CommandFailed {
                executable,
                context,
                status,
                stdout,
                stderr,
            }) => Ok(MemberCommandOutcome::Failed(format!(
                "command `{}` failed while {context}: status={status}\nstdout:\n{stdout}\nstderr:\n{stderr}",
                executable.display()
            ))),
            Err(err) => Err(err),
        }
    }

    fn member_published_port(&self, member: ClusterMember, port: &str) -> Result<u16> {
        let container_id = self.docker.compose_container_id(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            member.service_name(),
        )?;
        self.docker.published_host_port(container_id.as_str(), port)
    }

    fn ca_cert_path(&self) -> PathBuf {
        self.materialized_dir.join("configs/tls/ca.crt")
    }

    fn read_token_path(&self) -> PathBuf {
        self.materialized_dir.join("secrets/api-read-token")
    }

    fn admin_token_path(&self) -> PathBuf {
        self.materialized_dir.join("secrets/api-admin-token")
    }

    fn observer_cert_path(&self) -> PathBuf {
        self.materialized_dir.join("configs/tls/observer.crt")
    }

    fn observer_key_path(&self) -> PathBuf {
        self.materialized_dir.join("configs/tls/observer.key")
    }

    fn host_observer_config_path(&self, member: ClusterMember) -> PathBuf {
        self.materialized_dir
            .join("configs/observer")
            .join(format!("{}-pgtm.toml", member.service_name()))
    }

    fn materialize_host_observer_config(&self, member: ClusterMember) -> Result<PathBuf> {
        let published_api_port = self.member_published_port(member, "8443/tcp")?;
        let config_path = self.host_observer_config_path(member);
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|source| HarnessError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let rendered = render_host_observer_config(
            member,
            SocketAddr::from(([127, 0, 0, 1], published_api_port)),
            self.ca_cert_path().as_path(),
            self.read_token_path().as_path(),
            self.admin_token_path().as_path(),
            self.observer_cert_path().as_path(),
            self.observer_key_path().as_path(),
        );
        fs::write(config_path.as_path(), rendered).map_err(|source| HarnessError::Io {
            path: config_path.clone(),
            source,
        })?;
        Ok(config_path)
    }
}

fn resolve_pgtm_binary() -> Result<PathBuf> {
    let env_candidate = std::env::var_os("CARGO_BIN_EXE_pgtm")
        .map(PathBuf::from)
        .filter(|path| path.exists());
    let candidate = match env_candidate {
        Some(path) => path,
        None => {
            let settings = harness_settings()?;
            configured_executable(
                settings.pgtm.executable_candidates.as_slice(),
                "pgtm.executable_candidates",
                "pgtm",
            )?
        }
    };
    process::ensure_absolute_executable(candidate.as_path())?;
    Ok(candidate)
}

fn extract_state_command_output(output: CommandOutputDto) -> Result<StateCommandOutputDto> {
    match output {
        CommandOutputDto::State { output } => Ok(*output),
        other => Err(HarnessError::message(format!(
            "expected `pgtm status --json` output, observed command payload `{}`",
            command_label(&other)
        ))),
    }
}

fn extract_switchover_output(output: CommandOutputDto) -> Result<AcceptedResponse> {
    match output {
        CommandOutputDto::Switchover { output } => Ok(output),
        other => Err(HarnessError::message(format!(
            "expected `pgtm switchover request --json` output, observed command payload `{}`",
            command_label(&other)
        ))),
    }
}

fn command_label(output: &CommandOutputDto) -> &'static str {
    match output {
        CommandOutputDto::State { .. } => "state",
        CommandOutputDto::Primary { .. } => "primary",
        CommandOutputDto::Replicas { .. } => "replicas",
        CommandOutputDto::Switchover { .. } => "switchover",
        CommandOutputDto::ReloadCertificates { .. } => "reload_certificates",
    }
}

fn host_postgres_dsn(
    member: ClusterMember,
    port: u16,
    ca_cert_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> String {
    [
        format!("host={}", member.service_name()),
        "hostaddr=127.0.0.1".to_string(),
        format!("port={port}"),
        "user=postgres".to_string(),
        "dbname=postgres".to_string(),
        "sslmode=verify-full".to_string(),
        format!("sslrootcert={}", ca_cert_path.display()),
        format!("sslcert={}", observer_cert_path.display()),
        format!("sslkey={}", observer_key_path.display()),
    ]
    .join(" ")
}

fn render_host_observer_config(
    member: ClusterMember,
    resolve_to: SocketAddr,
    ca_cert_path: &Path,
    read_token_path: &Path,
    admin_token_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> String {
    format!(
        r#"[api]
base_url = "https://{}:{}"
expected_transport = "https"
resolve_to = "{resolve_to}"
auth = {{ type = "role_tokens", tokens = {{ read_token = {{ type = "file", path = "{}" }}, admin_token = {{ type = "file", path = "{}" }} }} }}
tls = {{ ca_cert = {{ path = "{}" }}, identity = {{ cert = {{ path = "{}" }}, key = {{ type = "file", path = "{}" }} }} }}

[postgres.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ type = "file", path = "{}" }} }}
"#,
        member.service_name(),
        resolve_to.port(),
        read_token_path.display(),
        admin_token_path.display(),
        ca_cert_path.display(),
        observer_cert_path.display(),
        observer_key_path.display(),
        ca_cert_path.display(),
        observer_cert_path.display(),
        observer_key_path.display(),
    )
}


===== tests/ha/support/mod.rs =====
mod config;
pub mod docker;
mod error;
pub mod faults;
pub mod givens;
mod invariant;
pub mod observer;
mod process;
pub mod runner;
pub mod steps;
pub mod timeouts;
pub mod topology;
pub mod world;

use std::sync::{Mutex, OnceLock};

use cucumber::{writer, World as _, WriterExt as _};
use futures::FutureExt as _;

use crate::support::{
    error::{HarnessError, Result},
    world::HaWorld,
};

#[derive(Clone, Debug)]
pub struct FeatureMetadata {
    pub feature_name: String,
}

static FEATURE_METADATA: OnceLock<FeatureMetadata> = OnceLock::new();
static CLEANUP_ERRORS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

// This runner is intentionally independent from the legacy HA harness so the old
// `tests/ha` and `src/test_harness/ha_e2e` flows can be deleted later.
pub async fn run_feature(
    feature_name: &str,
    feature_path: &str,
) -> std::result::Result<(), String> {
    install_context(feature_name, feature_path).map_err(|err| err.to_string())?;

    let writer = HaWorld::cucumber()
        .before(|_, _, scenario, world| {
            async move {
                world.reset();
                world.set_scenario_name(scenario.name.clone());
            }
            .boxed_local()
        })
        .after(|_, _, _, _, world| {
            async move {
                if let Some(world) = world {
                    if let Err(err) = world.cleanup() {
                        record_cleanup_error(err.to_string());
                    }
                }
            }
            .boxed_local()
        })
        .max_concurrent_scenarios(1)
        .with_writer(writer::Basic::stdout().summarized())
        .with_default_cli()
        .run(feature_path)
        .await;

    let stats_error = summarize_result(writer.scenarios_stats(), writer.steps_stats()).err();
    let cleanup_error = cleanup_recorded_errors().err();

    match (stats_error, cleanup_error) {
        (None, None) => Ok(()),
        (Some(stats), None) => Err(stats.to_string()),
        (None, Some(cleanup)) => Err(cleanup.to_string()),
        (Some(stats), Some(cleanup)) => Err(format!("{stats}\ncleanup also failed: {cleanup}")),
    }
}

pub fn feature_metadata() -> Result<&'static FeatureMetadata> {
    FEATURE_METADATA
        .get()
        .ok_or_else(|| HarnessError::message("feature metadata has not been initialized"))
}

fn install_context(feature_name: &str, _feature_path: &str) -> Result<()> {
    FEATURE_METADATA
        .set(FeatureMetadata {
            feature_name: feature_name.to_string(),
        })
        .map_err(|_| HarnessError::message("feature metadata was already initialized"))?;
    Ok(())
}

fn summarize_result(
    scenario_stats: &cucumber::writer::summarize::Stats,
    step_stats: &cucumber::writer::summarize::Stats,
) -> Result<()> {
    if scenario_stats.total() == 0 {
        return Err(HarnessError::message("cucumber executed zero scenarios"));
    }
    if scenario_stats.failed > 0 || step_stats.failed > 0 {
        return Err(HarnessError::message(format!(
            "cucumber feature failed: scenarios_failed={} steps_failed={}",
            scenario_stats.failed, step_stats.failed
        )));
    }
    if scenario_stats.skipped > 0 || step_stats.skipped > 0 {
        return Err(HarnessError::message(format!(
            "cucumber feature skipped steps unexpectedly: scenarios_skipped={} steps_skipped={}",
            scenario_stats.skipped, step_stats.skipped
        )));
    }
    Ok(())
}

fn cleanup_recorded_errors() -> Result<()> {
    let recorded = CLEANUP_ERRORS.get_or_init(|| Mutex::new(Vec::new()));
    let errors = {
        let mut guard = recorded
            .lock()
            .map_err(|_| HarnessError::message("cleanup error registry mutex was poisoned"))?;
        std::mem::take(&mut *guard)
    };

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HarnessError::message(errors.join("\n")))
    }
}

fn record_cleanup_error(error: String) {
    let recorded = CLEANUP_ERRORS.get_or_init(|| Mutex::new(Vec::new()));
    match recorded.lock() {
        Ok(mut guard) => guard.push(error),
        Err(poisoned) => poisoned.into_inner().push(error),
    }
}
