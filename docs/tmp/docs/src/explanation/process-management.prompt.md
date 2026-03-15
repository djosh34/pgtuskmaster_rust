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

docs/src/explanation/process-management.md

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
docs/src/reference/dcs-state-model.md
docs/src/reference/ha-decisions.md
docs/src/reference/http-api.md
docs/src/reference/overview.md
docs/src/reference/pgtm-cli.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/runtime-configuration.md
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
    - [HTTP API](reference/http-api.md)
    - [HA Decisions](reference/ha-decisions.md)
    - [DCS State Model](reference/dcs-state-model.md)
    - [pgtm CLI](reference/pgtm-cli.md)
    - [pgtuskmaster CLI](reference/pgtuskmaster-cli.md)
    - [Runtime Configuration](reference/runtime-configuration.md)



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
members = ["crates/pgtuskmaster_test_support"]

[features]
default = []
internal-test-support = []

[dependencies]
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
src/logging/event.rs
src/logging/mod.rs
src/logging/postgres_ingest.rs
src/logging/raw_record.rs
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
tests/ha/features/ha_all_dcs_services_stopped_on_three_etcd_enters_safe_degraded_mode_and_fences_post_cutoff_writes/ha_all_dcs_services_stopped_on_three_etcd_enters_safe_degraded_mode_and_fences_post_cutoff_writes.feature
tests/ha/features/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins.feature
tests/ha/features/ha_basebackup_clone_blocked_then_unblocked_replica_recovers/ha_basebackup_clone_blocked_then_unblocked_replica_recovers.feature
tests/ha/features/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum.feature
tests/ha/features/ha_dcs_and_api_faults_then_healed_cluster_converges/ha_dcs_and_api_faults_then_healed_cluster_converges.feature
tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature
tests/ha/features/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes.feature
tests/ha/features/ha_lagging_replica_is_not_promoted_during_failover/ha_lagging_replica_is_not_promoted_during_failover.feature
tests/ha/features/ha_non_primary_api_isolated_primary_stays_primary/ha_non_primary_api_isolated_primary_stays_primary.feature
tests/ha/features/ha_old_primary_partitioned_from_majority_majority_elects_new_primary/ha_old_primary_partitioned_from_majority_majority_elects_new_primary.feature
tests/ha/features/ha_old_primary_partitioned_from_majority_on_three_etcd_majority_elects_new_primary/ha_old_primary_partitioned_from_majority_on_three_etcd_majority_elects_new_primary.feature
tests/ha/features/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover.feature
tests/ha/features/ha_planned_switchover_changes_primary_cleanly/ha_planned_switchover_changes_primary_cleanly.feature
tests/ha/features/ha_planned_switchover_with_concurrent_writes/ha_planned_switchover_with_concurrent_writes.feature
tests/ha/features/ha_primary_killed_custom_roles_survive_rejoin/ha_primary_killed_custom_roles_survive_rejoin.feature
tests/ha/features/ha_primary_killed_then_rejoins_as_replica/ha_primary_killed_then_rejoins_as_replica.feature
tests/ha/features/ha_primary_killed_with_concurrent_writes/ha_primary_killed_with_concurrent_writes.feature
tests/ha/features/ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers/ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers.feature
tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature
tests/ha/features/ha_repeated_failovers_preserve_single_primary/ha_repeated_failovers_preserve_single_primary.feature
tests/ha/features/ha_replica_flapped_primary_stays_primary/ha_replica_flapped_primary_stays_primary.feature
tests/ha/features/ha_replica_loses_local_etcd_on_three_etcd_does_not_become_primary_and_primary_stays_primary/ha_replica_loses_local_etcd_on_three_etcd_does_not_become_primary_and_primary_stays_primary.feature
tests/ha/features/ha_replica_partitioned_from_majority_on_three_etcd_primary_stays_primary/ha_replica_partitioned_from_majority_on_three_etcd_primary_stays_primary.feature
tests/ha/features/ha_replica_partitioned_from_majority_primary_stays_primary/ha_replica_partitioned_from_majority_primary_stays_primary.feature
tests/ha/features/ha_replica_stopped_primary_stays_primary/ha_replica_stopped_primary_stays_primary.feature
tests/ha/features/ha_replication_path_isolated_then_healed_replicas_catch_up/ha_replication_path_isolated_then_healed_replicas_catch_up.feature
tests/ha/features/ha_rewind_fails_then_basebackup_rejoins_old_primary/ha_rewind_fails_then_basebackup_rejoins_old_primary.feature
tests/ha/features/ha_targeted_switchover_promotes_requested_replica/ha_targeted_switchover_promotes_requested_replica.feature
tests/ha/features/ha_targeted_switchover_to_degraded_replica_is_rejected/ha_targeted_switchover_to_degraded_replica_is_rejected.feature
tests/ha/features/ha_two_nodes_stopped_on_three_etcd_lone_survivor_never_keeps_primary/ha_two_nodes_stopped_on_three_etcd_lone_survivor_never_keeps_primary.feature
tests/ha/features/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken.feature
tests/ha/features/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum.feature
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
tests/ha/support/mod.rs
tests/ha/support/observer/mod.rs
tests/ha/support/observer/pgtm.rs
tests/ha/support/observer/sql.rs
tests/ha/support/process/mod.rs
tests/ha/support/runner/mod.rs
tests/ha/support/steps/mod.rs
tests/ha/support/timeouts/mod.rs
tests/ha/support/topology.rs
tests/ha/support/workload/mod.rs
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
docs/src/reference/dcs-state-model.md
docs/src/reference/ha-decisions.md
docs/src/reference/http-api.md
docs/src/reference/overview.md
docs/src/reference/pgtm-cli.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md
docs/src/tutorial/overview.md
docs/src/tutorial/performing-switchover.md
docs/src/tutorial/validating-cluster-behavior.md
docs/tmp/docs/src/explanation/process-management.prompt.md
docs/tmp/docs/src/explanation/trust-model.prompt.md
docs/tmp/verbose_extra_context/managed-postgres-roles.md
docs/tmp/verbose_extra_context/process-logging-boundary.md
docs/tmp/verbose_extra_context/trust-model.md


===== docs/src/explanation/process-management.md =====
# Process Management and Execution Domain

Process management is the execution boundary between the HA reconciler and the operating system. The HA side decides what should happen next. The process domain turns that decision into concrete PostgreSQL subprocess work, records the outcome, and publishes state for the rest of the node.

## Startup Boundary

The narrowed startup rewrite moved process-specific startup policy behind `ProcessRuntimePlan` and `process::startup::bootstrap(...)`.

`ProcessRuntimePlan::from_config(...)` projects the parts of `RuntimeConfig` that the process and pginfo domains need repeatedly:

- managed PostgreSQL paths and listen port
- replication-source defaults for replicator and rewinder jobs
- connection defaults such as dbname, SSL mode, CA path, and connect timeout

`ProcessRuntimePlan::ensure_start_paths()` also moved out of `runtime/node.rs`. It creates the data-dir parent, data dir, socket dir, and log parent before workers start. On Unix it additionally sets `0o700` permissions on the data directory.

At runtime composition level, `src/runtime/node.rs` now creates the plan once, prepares the paths once, and passes the typed plan into the owning startup modules instead of rebuilding loose strings and paths across domains.

## Worker Context Shape

`ProcessWorkerCtx` is no longer a flat startup bag. It groups concerns into narrower ADTs:

- `cadence`: worker poll interval and time source
- `config`: process-level timeout and binary configuration
- `identity`: the local `MemberId`
- `observed`: live `RuntimeConfig` and `DcsView` subscribers
- `plan`: the stable `ProcessRuntimePlan`
- `state_channel`: current `ProcessState`, publisher, and last rejection
- `control`: the inbox plus the optional active runtime
- `runtime`: logging, subprocess-output capture flag, and command runner

That split keeps the startup boundary smaller and makes cross-domain dependencies more explicit. The worker reads local identity and long-lived runtime defaults from typed bundles instead of from many unrelated top-level fields.

## Intent Flow

The HA reconciler never spawns a subprocess directly. It emits `ProcessIntent` values such as:

- `Bootstrap`
- `ProvisionReplica(BaseBackup | PgRewind)`
- `Start(Primary | DetachedStandby | Replica)`
- `Promote`
- `Demote(Fast | Immediate)`

`src/ha/process_dispatch.rs` converts each intent into a `ProcessIntentRequest` with a deterministic `JobId` built from scope, member id, HA tick, action index, and intent label. That request is sent through the process worker inbox.

```mermaid
flowchart LR
    A[HA reconcile] --> B[ProcessIntent]
    B --> C[process_dispatch]
    C --> D[ProcessIntentRequest<br/>deterministic JobId]
    D --> E[Process worker inbox]
    E --> F[Materialize execution request]
    F --> G[Build command spec]
    G --> H[Spawn PostgreSQL tool process]
    H --> I[Drain output and poll exit]
    I --> J[Publish ProcessState and JobOutcome]
```

If the worker is already busy, the new request is rejected without starting a second job. That rejection is recorded in `state_channel.last_rejection` and logged as a worker event.

## Materialization and Validation

The process worker turns `ProcessIntentRequest` into a concrete `ProcessExecutionRequest` inside `materialize_execution_request(...)`.

For replica-provisioning paths, materialization reads the latest DCS view and validates the chosen leader before building conninfo:

- the source member must not be `self`
- the advertised PostgreSQL host must be non-empty
- the source member must currently present as a primary in DCS

Those checks live in `src/process/source.rs` and use the typed replication-source defaults stored in `ProcessRuntimePlan`. That keeps replication-source policy in the process domain instead of leaving it spread across HA and runtime startup code.

The same materialization step also converts start intents into concrete PostgreSQL start specs, including detached-standby and replica-start managed configuration.

## Job Lifecycle and Timeouts

`ProcessState` exposes two high-level states:

- `Idle { worker, last_outcome }`
- `Running { worker, active }`

Internally, `ActiveRuntime` holds the execution request, deadline, process handle, and structured log identity for the running job.

Timeouts are enforced by deadline checks inside `tick_active_job(...)`. Different execution kinds resolve to different timeout defaults from `ProcessConfig`:

- bootstrap, basebackup, promote, and start-postgres use the bootstrap timeout unless the spec overrides it
- pg_rewind uses the pg_rewind timeout unless overridden
- demote uses the fencing timeout unless overridden

When the deadline is exceeded, the worker logs a timeout event, calls the process handle cancellation path, drains any remaining output, and transitions back to idle. In the current implementation that cancellation path is kill-based: `TokioProcessHandle::cancel()` uses `start_kill()` followed by `wait()`. A successful cancellation produces `JobOutcome::Timeout`; a cancellation failure becomes `JobOutcome::Failure`.

Subprocess output is drained during execution and again during shutdown paths. When `logging.capture_subprocess_output` is enabled, the process startup bundle projects that setting into `ProcessRuntime.capture_subprocess_output`, and stdout/stderr lines cross into the logging subsystem as typed subprocess events. The logging package then serializes those events into final log records tagged with the job identity.

## PostgreSQL Preflight Safety

The start-postgres path does extra preflight work before spawning `pg_ctl start`.

- It checks `postmaster.pid` in the configured data directory.
- It verifies whether that PID still exists and, on Unix, whether `/proc/<pid>/cmdline` looks like a PostgreSQL postmaster for the same data directory.
- It checks the PostgreSQL socket lock file for the configured port.
- If the PID or socket-lock evidence is stale, it removes the stale files before continuing.
- If PostgreSQL already appears to be running for that data directory or port, the start job becomes a no-op success instead of spawning another process.

This keeps the start path crash-tolerant and reduces false-positive "already running" failures after unclean shutdowns.

## Integration with PgInfo and API

The pginfo domain now shares the same `ProcessRuntimePlan` at startup rather than rebuilding its local socket target in `runtime/node.rs`. `PgProbeTarget::local_from_config(...)` derives the local probe conninfo from the runtime config plus the process plan, so the process and pginfo domains agree on the managed socket directory and port.

The API domain no longer reaches into process startup details either. It consumes published process state through its live observed-state bundle. During startup, the API can stay in `ApiObservedState::Unavailable` until the full live subscriber set is ready, which avoids pretending that partially wired state is already live.

## Why This Boundary Is Better

The rewrite makes `src/runtime/node.rs` a smaller composition root:

- runtime validates top-level config and boots global services
- process startup owns process-specific path preparation and runtime projection
- pginfo startup owns its local probe target
- HA sends typed intents instead of process commands
- API consumes published state instead of process internals

That boundary reduces startup duplication, shrinks the number of raw fields runtime must know about, and keeps process execution policy close to the code that actually launches and supervises PostgreSQL subprocesses.


===== src/process/worker.rs =====
use std::{fs, path::Path, process::Stdio};

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::mpsc::error::TryRecvError,
};

use crate::{
    config::{PostgresBinaryName, ProcessConfig, RoleAuthConfig, RuntimeConfig},
    dcs::{ClusterMemberView, DcsView},
    pginfo::state::render_pg_conninfo,
    postgres_managed::{inspect_managed_recovery_state, materialize_managed_postgres_config},
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::postmaster::{
        lookup_managed_postmaster, ManagedPostmasterError, ManagedPostmasterTarget,
    },
    state::{JobId, MemberId, UnixMillis, WorkerError, WorkerStatus},
};

use super::{
    jobs::{
        ActiveJob, ActiveJobKind, DemoteSpec, PostgresStartIntent, PostgresStartMode,
        ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar, ProcessError, ProcessExit,
        ProcessHandle, ProcessIntent, ProcessLogIdentity, ProcessOutputLine, ProcessOutputStream,
        PromoteSpec, ProcessJobKind, ReplicaProvisionIntent,
    },
    log_event::{
        CapturedStream, ProcessExecutionIdentity, ProcessJobIdentity, ProcessLogEvent,
        ProcessLogOrigin, SubprocessLogEvent,
    },
    source::{basebackup_source_from_member, rewind_source_from_member},
    state::{
        ActiveRuntime, JobOutcome, ProcessExecutionKind, ProcessExecutionRequest,
        ProcessIntentRequest, ProcessJobRejection, ProcessState, ProcessWorkerCtx,
    },
};

const PROCESS_OUTPUT_READ_CHUNK_BYTES: usize = 8192;
const PROCESS_OUTPUT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);
const PROCESS_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
const PG_CTL_DEFAULT_WAIT_SECONDS: u64 = 30;

#[derive(Default)]
pub(crate) struct TokioCommandRunner;

fn process_job_identity(job_id: &JobId, job_kind: ProcessJobKind) -> ProcessJobIdentity {
    ProcessJobIdentity {
        job_id: job_id.0.clone(),
        kind: job_kind,
    }
}

fn process_job_kind_from_intent(intent: &ProcessIntent) -> ProcessJobKind {
    match intent {
        ProcessIntent::Bootstrap => ProcessJobKind::Bootstrap,
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { .. }) => {
            ProcessJobKind::BaseBackup
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { .. }) => {
            ProcessJobKind::PgRewind
        }
        ProcessIntent::Start(PostgresStartIntent::Primary) => ProcessJobKind::StartPrimary,
        ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => {
            ProcessJobKind::StartDetachedStandby
        }
        ProcessIntent::Start(PostgresStartIntent::Replica { .. }) => ProcessJobKind::StartReplica,
        ProcessIntent::Promote => ProcessJobKind::Promote,
        ProcessIntent::Demote(_) => ProcessJobKind::Demote,
    }
}

fn process_job_kind_from_execution(kind: &ProcessExecutionKind) -> ProcessJobKind {
    match kind {
        ProcessExecutionKind::Bootstrap(_) => ProcessJobKind::Bootstrap,
        ProcessExecutionKind::BaseBackup(_) => ProcessJobKind::BaseBackup,
        ProcessExecutionKind::PgRewind(_) => ProcessJobKind::PgRewind,
        ProcessExecutionKind::Promote(_) => ProcessJobKind::Promote,
        ProcessExecutionKind::Demote(_) => ProcessJobKind::Demote,
        ProcessExecutionKind::StartPostgres(_) => ProcessJobKind::StartPostgres,
    }
}

fn process_execution_identity(identity: &ProcessLogIdentity) -> ProcessExecutionIdentity {
    ProcessExecutionIdentity {
        job: process_job_identity(&identity.job_id, identity.job_kind),
        binary: identity.binary.clone(),
    }
}

struct TokioProcessHandle {
    child: Child,
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
    stdout_pending: Vec<u8>,
    stderr_pending: Vec<u8>,
    stdout_eof: bool,
    stderr_eof: bool,
}

impl ProcessHandle for TokioProcessHandle {
    fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
        let status = self
            .child
            .try_wait()
            .map_err(|err| ProcessError::SpawnFailure {
                binary: "process-child".to_string(),
                message: err.to_string(),
            })?;

        Ok(status.map(|exit| {
            if exit.success() {
                ProcessExit::Success
            } else {
                ProcessExit::Failure { code: exit.code() }
            }
        }))
    }

    fn drain_output<'a>(
        &'a mut self,
        max_bytes: usize,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<super::jobs::ProcessOutputLine>, ProcessError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if max_bytes == 0 {
                return Ok(Vec::new());
            }

            let mut out = Vec::new();
            let mut remaining = max_bytes;
            drain_one_stream(
                &mut out,
                &mut remaining,
                super::jobs::ProcessOutputStream::Stdout,
                &mut self.stdout,
                &mut self.stdout_pending,
                &mut self.stdout_eof,
            )
            .await;
            drain_one_stream(
                &mut out,
                &mut remaining,
                super::jobs::ProcessOutputStream::Stderr,
                &mut self.stderr,
                &mut self.stderr_pending,
                &mut self.stderr_eof,
            )
            .await;
            Ok(out)
        })
    }

    fn cancel<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>>
    {
        Box::pin(async move {
            if self
                .child
                .try_wait()
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?
                .is_some()
            {
                return Ok(());
            }

            self.child
                .start_kill()
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?;
            self.child
                .wait()
                .await
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?;
            Ok(())
        })
    }
}

impl super::jobs::ProcessCommandRunner for TokioCommandRunner {
    fn spawn(&mut self, spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError> {
        let ProcessCommandSpec {
            program,
            args,
            env,
            capture_output,
            log_identity: _,
        } = spec;
        let binary = program.display().to_string();
        if !program.is_absolute() {
            return Err(ProcessError::InvalidSpec(format!(
                "program must be an absolute path, got `{}`",
                program.display()
            )));
        }

        let mut command = Command::new(&program);
        command.args(args).stdin(Stdio::null());
        for var in env {
            let value = var.value.resolve_string_for_key(var.key.as_str())?;
            command.env(var.key, value);
        }
        if capture_output {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        } else {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }

        let mut child = command.spawn().map_err(|err| ProcessError::SpawnFailure {
            binary,
            message: err.to_string(),
        })?;

        let stdout = if capture_output {
            child.stdout.take()
        } else {
            None
        };
        let stderr = if capture_output {
            child.stderr.take()
        } else {
            None
        };

        Ok(Box::new(TokioProcessHandle {
            child,
            stdout,
            stderr,
            stdout_pending: Vec::new(),
            stderr_pending: Vec::new(),
            stdout_eof: false,
            stderr_eof: false,
        }))
    }
}

async fn drain_one_stream(
    out: &mut Vec<super::jobs::ProcessOutputLine>,
    remaining: &mut usize,
    stream: super::jobs::ProcessOutputStream,
    handle: &mut Option<impl AsyncRead + Unpin>,
    pending: &mut Vec<u8>,
    eof: &mut bool,
) {
    if *remaining == 0 || *eof {
        return;
    }
    let Some(handle) = handle.as_mut() else {
        *eof = true;
        return;
    };

    let mut buf = vec![0u8; PROCESS_OUTPUT_READ_CHUNK_BYTES];
    loop {
        if *remaining == 0 {
            break;
        }
        let chunk_len = buf.len().min(*remaining);
        let read_result = tokio::time::timeout(
            PROCESS_OUTPUT_READ_TIMEOUT,
            handle.read(&mut buf[..chunk_len]),
        )
        .await;
        let read_outcome = match read_result {
            Ok(Ok(n)) => Ok(n),
            Ok(Err(err)) => Err(err),
            Err(_) => {
                // No data quickly available.
                break;
            }
        };

        match read_outcome {
            Ok(0) => {
                *eof = true;
                if !pending.is_empty() {
                    out.push(super::jobs::ProcessOutputLine {
                        stream,
                        bytes: std::mem::take(pending),
                    });
                }
                break;
            }
            Ok(n) => {
                pending.extend_from_slice(&buf[..n]);
                *remaining = remaining.saturating_sub(n);
                while let Some(pos) = pending.iter().position(|b| *b == b'\n') {
                    let mut line = pending.drain(..=pos).collect::<Vec<u8>>();
                    if let Some(b'\n') = line.last() {
                        line.pop();
                    }
                    if let Some(b'\r') = line.last() {
                        line.pop();
                    }
                    out.push(super::jobs::ProcessOutputLine {
                        stream,
                        bytes: line,
                    });
                }
            }
            Err(err) => {
                *eof = true;
                out.push(super::jobs::ProcessOutputLine {
                    stream,
                    bytes: format!("stdio read error: {err}").into_bytes(),
                });
                break;
            }
        }
    }
}

fn can_accept_job(state: &ProcessState) -> bool {
    matches!(state, ProcessState::Idle { .. })
}

pub(crate) async fn run(mut ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    ctx.runtime
        .log
        .send(ProcessLogEvent::WorkerRunStarted {
            origin: ProcessLogOrigin::Run,
            capture_subprocess_output: ctx.runtime.capture_subprocess_output,
        })
        .map_err(|err| {
            WorkerError::Message(format!("process worker start log send failed: {err}"))
        })?;
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.cadence.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    match ctx.control.inbox.try_recv() {
        Ok(request) => {
            ctx.runtime
                .log
                .send(ProcessLogEvent::RequestReceived {
                    origin: ProcessLogOrigin::StepOnce,
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_intent(&request.intent),
                    ),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process request log send failed: {err}"))
                })?;
            start_job(ctx, request).await?;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            if !ctx.control.inbox_disconnected_logged {
                ctx.control.inbox_disconnected_logged = true;
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::InboxDisconnected {
                        origin: ProcessLogOrigin::StepOnce,
                    })
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process inbox disconnected log send failed: {err}"
                        ))
                    })?;
            }
        }
    }

    tick_active_job(ctx).await
}

fn pid_is_postgres_process(pid: u32) -> Result<bool, ProcessError> {
    #[cfg(unix)]
    {
        let cmdline_path = std::path::PathBuf::from(format!("/proc/{pid}/cmdline"));
        let cmdline = match fs::read(&cmdline_path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => {
                return Err(ProcessError::InvalidSpec(format!(
                    "read {} failed: {err}",
                    cmdline_path.display()
                )));
            }
        };
        Ok(cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg))
            .any(|arg| {
                std::path::Path::new(arg.as_ref())
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| matches!(name, "postgres" | "postmaster"))
                    .unwrap_or(false)
            }))
    }
    #[cfg(not(unix))]
    {
        Ok(true)
    }
}

fn remove_file_best_effort(path: &Path) -> Result<(), ProcessError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(ProcessError::InvalidSpec(format!(
            "remove file {} failed: {err}",
            path.display()
        ))),
    }
}

fn postgres_socket_paths(socket_dir: &Path, port: u16) -> (std::path::PathBuf, std::path::PathBuf) {
    let socket_file = socket_dir.join(format!(".s.PGSQL.{port}"));
    let lock_file = socket_dir.join(format!(".s.PGSQL.{port}.lock"));
    (socket_file, lock_file)
}

fn parse_postgres_socket_lock_pid(lock_file: &Path) -> Result<Option<u32>, ProcessError> {
    let contents = match fs::read_to_string(lock_file) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(ProcessError::InvalidSpec(format!(
                "read postgres socket lock {} failed: {err}",
                lock_file.display()
            )));
        }
    };
    let Some(first_line) = contents.lines().next() else {
        return Ok(None);
    };
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed.parse::<u32>().map(Some).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "parse postgres socket lock pid '{}' in {} failed: {err}",
            trimmed,
            lock_file.display()
        ))
    })
}

fn cleanup_postgres_socket_files(socket_dir: &Path, port: u16) -> Result<(), ProcessError> {
    let (socket_file, lock_file) = postgres_socket_paths(socket_dir, port);
    remove_file_best_effort(&socket_file)?;
    remove_file_best_effort(&lock_file)?;
    Ok(())
}

fn start_postgres_preflight_is_already_running(
    data_dir: &Path,
    socket_dir: &Path,
    port: u16,
) -> Result<bool, ProcessError> {
    let pid_file = data_dir.join("postmaster.pid");
    if pid_file.exists() {
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
        match lookup_managed_postmaster(&target) {
            Ok(_postmaster) => return Ok(true),
            Err(
                ManagedPostmasterError::MissingPidFile { .. }
                | ManagedPostmasterError::PidNotRunning { .. }
                | ManagedPostmasterError::DataDirMismatch { .. },
            ) => {
                remove_file_best_effort(&pid_file)?;
                let opts_file = data_dir.join("postmaster.opts");
                remove_file_best_effort(&opts_file)?;
            }
            Err(err) => {
                return Err(ProcessError::InvalidSpec(format!(
                    "start postgres preflight managed postmaster lookup failed: {err}"
                )));
            }
        }
    }

    let (_, lock_file) = postgres_socket_paths(socket_dir, port);
    if let Some(pid) = parse_postgres_socket_lock_pid(&lock_file)? {
        if pid_is_postgres_process(pid)? {
            return Ok(true);
        }
    }

    cleanup_postgres_socket_files(socket_dir, port)?;
    Ok(false)
}

fn start_postgres_preflight_details(
    ctx: &ProcessWorkerCtx,
    intent: &ProcessIntent,
) -> Option<(std::path::PathBuf, std::path::PathBuf, u16)> {
    match intent {
        ProcessIntent::Start(
            PostgresStartIntent::Primary
            | PostgresStartIntent::DetachedStandby
            | PostgresStartIntent::Replica { .. },
        ) => {
            let runtime_config = ctx.observed.runtime_config.latest();
            Some((
                runtime_config.postgres.paths.data_dir.clone(),
                ctx.plan.postgres.paths.socket_dir.clone(),
                ctx.plan.postgres.port,
            ))
        }
        _ => None,
    }
}

pub(crate) async fn start_job(
    ctx: &mut ProcessWorkerCtx,
    request: ProcessIntentRequest,
) -> Result<(), WorkerError> {
    if !can_accept_job(&ctx.state_channel.current) {
        let now = current_time(ctx)?;
        let rejected_job_id = request.id.clone();
        ctx.state_channel.last_rejection = Some(ProcessJobRejection {
            id: rejected_job_id.clone(),
            error: ProcessError::Busy,
            rejected_at: now,
        });
        ctx.runtime
            .log
            .send(ProcessLogEvent::BusyRejected {
                origin: ProcessLogOrigin::StartJob,
                job: process_job_identity(
                    &rejected_job_id,
                    process_job_kind_from_intent(&request.intent),
                ),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process busy reject log send failed: {err}"))
            })?;
        return Ok(());
    }

    let now = current_time(ctx)?;
    if let Some((data_dir, socket_dir, port)) =
        start_postgres_preflight_details(ctx, &request.intent)
    {
        match start_postgres_preflight_is_already_running(
            data_dir.as_path(),
            socket_dir.as_path(),
            port,
        ) {
            Ok(true) => {
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::StartPostgresAlreadyRunning {
                        origin: ProcessLogOrigin::StartJob,
                        job: process_job_identity(&request.id, ProcessJobKind::StartPostgres),
                        data_dir: data_dir.display().to_string(),
                    })
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres noop log send failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Success {
                        id: request.id,
                        job_kind: active_kind_from_intent(&request.intent),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::StartPostgresPreflightFailed {
                        origin: ProcessLogOrigin::StartJob,
                        job: process_job_identity(&request.id, ProcessJobKind::StartPostgres),
                        error: error.to_string(),
                    })
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres preflight log send failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Failure {
                        id: request.id,
                        job_kind: active_kind_from_intent(&request.intent),
                        error,
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
        }
    }

    let execution_request = match materialize_execution_request(ctx, &request) {
        Ok(materialized) => materialized,
        Err(error) => {
            ctx.runtime
                .log
                .send(ProcessLogEvent::IntentMaterializationFailed {
                    origin: ProcessLogOrigin::StartJob,
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_intent(&request.intent),
                    ),
                    error: error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "process intent materialization log send failed: {err}"
                    ))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind_from_intent(&request.intent),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };
    let timeout_ms = timeout_for_kind(&execution_request.kind, &ctx.config);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));

    let command = match build_command(
        &ctx.config,
        &request.id,
        &execution_request.kind,
        ctx.runtime.capture_subprocess_output,
    ) {
        Ok(command) => command,
        Err(error) => {
            ctx.runtime
                .log
                .send(ProcessLogEvent::BuildCommandFailed {
                    origin: ProcessLogOrigin::StartJob,
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_execution(&execution_request.kind),
                    ),
                    error: error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "process build command log send failed: {err}"
                    ))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&execution_request.kind),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let log_identity = command.log_identity.clone();
    let handle = match ctx.runtime.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            ctx.runtime
                .log
                .send(ProcessLogEvent::SpawnFailed {
                    origin: ProcessLogOrigin::StartJob,
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_execution(&execution_request.kind),
                    ),
                    error: error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process spawn log send failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&execution_request.kind),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let active = ActiveJob {
        id: request.id.clone(),
        kind: active_kind(&execution_request.kind),
        started_at: now,
        deadline_at,
    };
    let started_execution = process_execution_identity(&log_identity);

    ctx.control.active_runtime = Some(ActiveRuntime {
        request: execution_request,
        deadline_at,
        handle,
        log_identity,
    });
    ctx.state_channel.current = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    ctx.runtime
        .log
        .send(ProcessLogEvent::Started {
            origin: ProcessLogOrigin::StartJob,
            execution: started_execution,
        })
        .map_err(|err| WorkerError::Message(format!("process job started log send failed: {err}")))?;
    publish_state(ctx)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut runtime = match ctx.control.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    match runtime
        .handle
        .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
        .await
    {
        Ok(lines) => {
            for line in lines {
                if let Err(err) = ctx.runtime.log.send(subprocess_log_event(
                    &runtime.log_identity,
                    line.clone(),
                )) {
                    ctx.runtime
                        .log
                        .send(ProcessLogEvent::OutputEmitFailed {
                            origin: ProcessLogOrigin::EmitSubprocessLine,
                            execution: process_execution_identity(&runtime.log_identity),
                            stream: captured_stream(line.stream),
                            bytes_len: line.bytes.len(),
                            error: err.to_string(),
                        })
                        .map_err(|send_err| {
                            WorkerError::Message(format!(
                                "process output emit failure log send failed: {send_err}"
                            ))
                        })?;
                }
            }
        }
        Err(err) => ctx.runtime
            .log
            .send(ProcessLogEvent::OutputDrainFailed {
                origin: ProcessLogOrigin::TickActiveJob,
                execution: process_execution_identity(&runtime.log_identity),
                error: err.to_string(),
            })
            .map_err(|send_err| {
                WorkerError::Message(format!(
                    "process output drain log send failed: {send_err}"
                ))
            })?,
    }
    if now.0 >= runtime.deadline_at.0 {
        ctx.runtime
            .log
            .send(ProcessLogEvent::Timeout {
                origin: ProcessLogOrigin::TickActiveJob,
                execution: process_execution_identity(&runtime.log_identity),
            })
            .map_err(|err| WorkerError::Message(format!("process timeout log send failed: {err}")))?;
        let cancel_result = runtime.handle.cancel().await;
        match runtime
            .handle
            .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
            .await
        {
            Ok(lines) => {
                for line in lines {
                    if let Err(err) = ctx.runtime.log.send(subprocess_log_event(
                        &runtime.log_identity,
                        line.clone(),
                    )) {
                        ctx.runtime
                            .log
                            .send(ProcessLogEvent::OutputEmitFailed {
                                origin: ProcessLogOrigin::EmitSubprocessLine,
                                execution: process_execution_identity(&runtime.log_identity),
                                stream: captured_stream(line.stream),
                                bytes_len: line.bytes.len(),
                                error: err.to_string(),
                            })
                            .map_err(|send_err| {
                                WorkerError::Message(format!(
                                    "process output emit failure log send failed: {send_err}"
                                ))
                            })?;
                    }
                }
            }
            Err(err) => ctx.runtime
                .log
                .send(ProcessLogEvent::OutputDrainFailed {
                    origin: ProcessLogOrigin::TickActiveJob,
                    execution: process_execution_identity(&runtime.log_identity),
                    error: err.to_string(),
                })
                .map_err(|send_err| {
                    WorkerError::Message(format!(
                        "process output drain log send failed: {send_err}"
                    ))
                })?,
        }
        let outcome = match cancel_result {
            Ok(()) => JobOutcome::Timeout {
                id: runtime.request.id,
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: runtime.request.id,
                job_kind: active_kind(&runtime.request.kind),
                error,
                finished_at: now,
            },
        };
        transition_to_idle(ctx, outcome, now)?;
        return Ok(());
    }

    let poll = runtime.handle.poll_exit();
    match poll {
        Ok(None) => {
            ctx.control.active_runtime = Some(runtime);
            Ok(())
        }
        Ok(Some(ProcessExit::Success)) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = ctx.runtime.log.send(subprocess_log_event(
                            &runtime.log_identity,
                            line.clone(),
                        )) {
                            ctx.runtime
                                .log
                                .send(ProcessLogEvent::OutputEmitFailed {
                                    origin: ProcessLogOrigin::EmitSubprocessLine,
                                    execution: process_execution_identity(&runtime.log_identity),
                                    stream: captured_stream(line.stream),
                                    bytes_len: line.bytes.len(),
                                    error: err.to_string(),
                                })
                                .map_err(|send_err| {
                                    WorkerError::Message(format!(
                                        "process output emit failure log send failed: {send_err}"
                                    ))
                                })?;
                        }
                    }
                }
                Err(err) => ctx.runtime
                    .log
                    .send(ProcessLogEvent::OutputDrainFailed {
                        origin: ProcessLogOrigin::TickActiveJob,
                        execution: process_execution_identity(&runtime.log_identity),
                        error: err.to_string(),
                    })
                    .map_err(|send_err| {
                        WorkerError::Message(format!(
                            "process output drain log send failed: {send_err}"
                        ))
                    })?,
            }
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Success {
                id: job_id,
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::ExitedSuccessfully {
                    origin: ProcessLogOrigin::TickActiveJob,
                    execution: process_execution_identity(&runtime.log_identity),
                })
                .map_err(|err| WorkerError::Message(format!("process exit log send failed: {err}")))?;
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = ctx.runtime.log.send(subprocess_log_event(
                            &runtime.log_identity,
                            line.clone(),
                        )) {
                            ctx.runtime
                                .log
                                .send(ProcessLogEvent::OutputEmitFailed {
                                    origin: ProcessLogOrigin::EmitSubprocessLine,
                                    execution: process_execution_identity(&runtime.log_identity),
                                    stream: captured_stream(line.stream),
                                    bytes_len: line.bytes.len(),
                                    error: err.to_string(),
                                })
                                .map_err(|send_err| {
                                    WorkerError::Message(format!(
                                        "process output emit failure log send failed: {send_err}"
                                    ))
                                })?;
                        }
                    }
                }
                Err(err) => ctx.runtime
                    .log
                    .send(ProcessLogEvent::OutputDrainFailed {
                        origin: ProcessLogOrigin::TickActiveJob,
                        execution: process_execution_identity(&runtime.log_identity),
                        error: err.to_string(),
                    })
                    .map_err(|send_err| {
                        WorkerError::Message(format!(
                            "process output drain log send failed: {send_err}"
                        ))
                    })?,
            }
            let exit_error = ProcessError::from_exit(exit);
            let outcome = JobOutcome::Failure {
                id: runtime.request.id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error: exit_error.clone(),
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::ExitedUnsuccessfully {
                    origin: ProcessLogOrigin::TickActiveJob,
                    execution: process_execution_identity(&runtime.log_identity),
                    error: exit_error.to_string(),
                })
                .map_err(|err| WorkerError::Message(format!("process exit log send failed: {err}")))?;
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = ctx.runtime.log.send(subprocess_log_event(
                            &runtime.log_identity,
                            line.clone(),
                        )) {
                            ctx.runtime
                                .log
                                .send(ProcessLogEvent::OutputEmitFailed {
                                    origin: ProcessLogOrigin::EmitSubprocessLine,
                                    execution: process_execution_identity(&runtime.log_identity),
                                    stream: captured_stream(line.stream),
                                    bytes_len: line.bytes.len(),
                                    error: err.to_string(),
                                })
                                .map_err(|send_err| {
                                    WorkerError::Message(format!(
                                        "process output emit failure log send failed: {send_err}"
                                    ))
                                })?;
                        }
                    }
                }
                Err(err) => ctx.runtime
                    .log
                    .send(ProcessLogEvent::OutputDrainFailed {
                        origin: ProcessLogOrigin::TickActiveJob,
                        execution: process_execution_identity(&runtime.log_identity),
                        error: err.to_string(),
                    })
                    .map_err(|send_err| {
                        WorkerError::Message(format!(
                            "process output drain log send failed: {send_err}"
                        ))
                    })?,
            }
            let outcome = JobOutcome::Failure {
                id: runtime.request.id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error,
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::PollFailed {
                    origin: ProcessLogOrigin::TickActiveJob,
                    execution: process_execution_identity(&runtime.log_identity),
                    error: outcome_error_string(&outcome),
                })
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "process poll failure log send failed: {err}"
                    ))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
    }
}

fn outcome_error_string(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { .. } => "success".to_string(),
        JobOutcome::Timeout { .. } => "timeout".to_string(),
        JobOutcome::Failure { error, .. } => error.to_string(),
    }
}

fn captured_stream(stream: ProcessOutputStream) -> CapturedStream {
    match stream {
        ProcessOutputStream::Stdout => CapturedStream::Stdout,
        ProcessOutputStream::Stderr => CapturedStream::Stderr,
    }
}

fn subprocess_log_event(
    identity: &ProcessLogIdentity,
    line: ProcessOutputLine,
) -> SubprocessLogEvent {
    SubprocessLogEvent {
        producer: crate::logging::LogProducer::PgTool,
        stream: captured_stream(line.stream),
        execution: process_execution_identity(identity),
        origin: ProcessLogOrigin::EmitSubprocessLine,
        bytes: line.bytes,
    }
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx,
    outcome: JobOutcome,
    _now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state_channel.current = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx)
}

fn publish_state(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    ctx.state_channel
        .publisher
        .publish(ctx.state_channel.current.clone())
        .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
    Ok(())
}

fn current_time(ctx: &mut ProcessWorkerCtx) -> Result<UnixMillis, WorkerError> {
    (ctx.cadence.now)()
}

pub(crate) fn system_now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

fn timeout_for_kind(kind: &ProcessExecutionKind, config: &ProcessConfig) -> u64 {
    match kind {
        ProcessExecutionKind::Bootstrap(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
        ProcessExecutionKind::BaseBackup(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
        ProcessExecutionKind::PgRewind(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.pg_rewind_ms)
        }
        ProcessExecutionKind::Promote(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
        ProcessExecutionKind::Demote(spec) => spec.timeout_ms.unwrap_or(config.timeouts.fencing_ms),
        ProcessExecutionKind::StartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
    }
}

fn active_kind(kind: &ProcessExecutionKind) -> ActiveJobKind {
    match kind {
        ProcessExecutionKind::Bootstrap(_) => ActiveJobKind::Bootstrap,
        ProcessExecutionKind::BaseBackup(_) => ActiveJobKind::BaseBackup,
        ProcessExecutionKind::PgRewind(_) => ActiveJobKind::PgRewind,
        ProcessExecutionKind::Promote(_) => ActiveJobKind::Promote,
        ProcessExecutionKind::Demote(_) => ActiveJobKind::Demote,
        ProcessExecutionKind::StartPostgres(spec) => match spec.mode {
            PostgresStartMode::Primary => ActiveJobKind::StartPrimary,
            PostgresStartMode::DetachedStandby => ActiveJobKind::StartDetachedStandby,
            PostgresStartMode::Replica => ActiveJobKind::StartReplica,
        },
    }
}

fn active_kind_from_intent(intent: &ProcessIntent) -> ActiveJobKind {
    match intent {
        ProcessIntent::Bootstrap => ActiveJobKind::Bootstrap,
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { .. }) => {
            ActiveJobKind::BaseBackup
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { .. }) => {
            ActiveJobKind::PgRewind
        }
        ProcessIntent::Promote => ActiveJobKind::Promote,
        ProcessIntent::Demote(_) => ActiveJobKind::Demote,
        ProcessIntent::Start(PostgresStartIntent::Primary) => ActiveJobKind::StartPrimary,
        ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => {
            ActiveJobKind::StartDetachedStandby
        }
        ProcessIntent::Start(PostgresStartIntent::Replica { .. }) => ActiveJobKind::StartReplica,
    }
}

fn build_command(
    config: &ProcessConfig,
    job_id: &JobId,
    kind: &ProcessExecutionKind,
    capture_output: bool,
) -> Result<ProcessCommandSpec, ProcessError> {
    match kind {
        ProcessExecutionKind::Bootstrap(spec) => {
            validate_non_empty_path("bootstrap.data_dir", &spec.data_dir)?;
            if spec.superuser.as_str().trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "bootstrap.superuser must not be empty".to_string(),
                ));
            }
            let program = resolve_process_binary(config, PostgresBinaryName::Initdb)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-A".to_string(),
                    "trust".to_string(),
                    "-U".to_string(),
                    spec.superuser.as_str().to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::BaseBackup(spec) => {
            validate_non_empty_path("basebackup.data_dir", &spec.data_dir)?;
            validate_non_empty_pg_connect_target(
                "basebackup.source_conninfo.target",
                &spec.source.conninfo.target,
            )?;
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = resolve_process_binary(config, PostgresBinaryName::PgBasebackup)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--dbname".to_string(),
                    render_pg_conninfo(&spec.source.conninfo),
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-Fp".to_string(),
                    "-Xs".to_string(),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::PgRewind(spec) => {
            validate_non_empty_path("pg_rewind.target_data_dir", &spec.target_data_dir)?;
            validate_non_empty_pg_connect_target(
                "pg_rewind.source_conninfo.target",
                &spec.source.conninfo.target,
            )?;
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = resolve_process_binary(config, PostgresBinaryName::PgRewind)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--target-pgdata".to_string(),
                    spec.target_data_dir.display().to_string(),
                    "--source-server".to_string(),
                    render_pg_conninfo(&spec.source.conninfo),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::Promote(spec) => {
            validate_non_empty_path("promote.data_dir", &spec.data_dir)?;
            let mut args = vec![
                "-D".to_string(),
                spec.data_dir.display().to_string(),
                "promote".to_string(),
                "-w".to_string(),
            ];
            if let Some(wait_seconds) = spec.wait_seconds {
                args.push("-t".to_string());
                args.push(wait_seconds.to_string());
            }
            let program = resolve_process_binary(config, PostgresBinaryName::PgCtl)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::Demote(spec) => {
            validate_non_empty_path("demote.data_dir", &spec.data_dir)?;
            let program = resolve_process_binary(config, PostgresBinaryName::PgCtl)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::StartPostgres(spec) => {
            validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("start_postgres.config_file", &spec.config_file)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            let wait_seconds = spec.wait_seconds.unwrap_or(PG_CTL_DEFAULT_WAIT_SECONDS);
            let option_tokens = vec![
                "-c".to_string(),
                format!("config_file={}", spec.config_file.display()),
            ];
            let options = render_pg_ctl_option_string(&option_tokens)?;
            let program = resolve_process_binary(config, PostgresBinaryName::PgCtl)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-l".to_string(),
                    spec.log_file.display().to_string(),
                    "-o".to_string(),
                    options,
                    "start".to_string(),
                    "-w".to_string(),
                    "-t".to_string(),
                    wait_seconds.to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
    }
}

fn resolve_process_binary(
    config: &ProcessConfig,
    binary: PostgresBinaryName,
) -> Result<std::path::PathBuf, ProcessError> {
    config
        .binaries
        .resolve_binary_path(binary)
        .map_err(ProcessError::InvalidSpec)
}

fn role_auth_env(auth: &RoleAuthConfig) -> Vec<ProcessEnvVar> {
    match auth {
        RoleAuthConfig::Password { password } => vec![ProcessEnvVar {
            key: "PGPASSWORD".to_string(),
            value: ProcessEnvValue::Secret(password.clone()),
        }],
    }
}

fn materialize_execution_request(
    ctx: &ProcessWorkerCtx,
    request: &ProcessIntentRequest,
) -> Result<ProcessExecutionRequest, ProcessError> {
    let runtime_config = ctx.observed.runtime_config.latest();
    let dcs = ctx.observed.dcs.latest();
    let kind = match &request.intent {
        ProcessIntent::Bootstrap => {
            wipe_data_dir(runtime_config.postgres.paths.data_dir.as_path())?;
            ProcessExecutionKind::Bootstrap(super::jobs::BootstrapSpec {
                data_dir: runtime_config.postgres.paths.data_dir.clone(),
                superuser: runtime_config
                    .postgres
                    .roles
                    .mandatory
                    .superuser
                    .username
                    .clone(),
                timeout_ms: None,
            })
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
            wipe_data_dir(runtime_config.postgres.paths.data_dir.as_path())?;
            let (source_member_id, source_member) = resolve_source_member(&dcs, leader)?;
            let source = basebackup_source_from_member(
                &ctx.identity.self_id,
                &ctx.plan,
                source_member_id,
                source_member,
            )
            .map_err(source_materialization_error)?;
            ProcessExecutionKind::BaseBackup(super::jobs::BaseBackupSpec {
                data_dir: runtime_config.postgres.paths.data_dir.clone(),
                source,
                timeout_ms: Some(runtime_config.process.timeouts.bootstrap_ms),
            })
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }) => {
            let (source_member_id, source_member) = resolve_source_member(&dcs, leader)?;
            let source = rewind_source_from_member(
                &ctx.identity.self_id,
                &ctx.plan,
                source_member_id,
                source_member,
            )
            .map_err(source_materialization_error)?;
            ProcessExecutionKind::PgRewind(super::jobs::PgRewindSpec {
                target_data_dir: runtime_config.postgres.paths.data_dir.clone(),
                source,
                timeout_ms: None,
            })
        }
        ProcessIntent::Start(PostgresStartIntent::Primary) => {
            let start_intent = primary_start_intent(&runtime_config)?;
            materialize_start_postgres(
                &runtime_config,
                &ctx.plan,
                PostgresStartMode::Primary,
                &start_intent,
            )?
        }
        ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => materialize_start_postgres(
            &runtime_config,
            &ctx.plan,
            PostgresStartMode::DetachedStandby,
            &ManagedPostgresStartIntent::detached_standby(),
        )?,
        ProcessIntent::Start(PostgresStartIntent::Replica { leader }) => {
            let start_intent = replica_start_intent(ctx, &runtime_config, &dcs, leader)?;
            materialize_start_postgres(
                &runtime_config,
                &ctx.plan,
                PostgresStartMode::Replica,
                &start_intent,
            )?
        }
        ProcessIntent::Promote => ProcessExecutionKind::Promote(PromoteSpec {
            data_dir: runtime_config.postgres.paths.data_dir.clone(),
            wait_seconds: None,
            timeout_ms: None,
        }),
        ProcessIntent::Demote(mode) => ProcessExecutionKind::Demote(DemoteSpec {
            data_dir: runtime_config.postgres.paths.data_dir.clone(),
            mode: mode.clone(),
            timeout_ms: None,
        }),
    };

    Ok(ProcessExecutionRequest {
        id: request.id.clone(),
        kind,
    })
}

fn primary_start_intent(
    runtime_config: &RuntimeConfig,
) -> Result<ManagedPostgresStartIntent, ProcessError> {
    let managed_recovery_state = inspect_managed_recovery_state(
        runtime_config.postgres.paths.data_dir.as_path(),
    )
    .map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "inspect managed recovery state for primary start failed: {err}"
        ))
    })?;
    if managed_recovery_state != crate::postgres_managed_conf::ManagedRecoverySignal::None {
        return Err(ProcessError::InvalidSpec(
            "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config".to_string(),
        ));
    }
    Ok(ManagedPostgresStartIntent::primary())
}

fn replica_start_intent(
    ctx: &ProcessWorkerCtx,
    runtime_config: &RuntimeConfig,
    dcs: &DcsView,
    leader: &crate::state::MemberId,
) -> Result<ManagedPostgresStartIntent, ProcessError> {
    let (source_member_id, source_member) = resolve_source_member(dcs, leader)?;
    let source = basebackup_source_from_member(
        &ctx.identity.self_id,
        &ctx.plan,
        source_member_id,
        source_member,
    )
    .map_err(source_materialization_error)?;
    Ok(ManagedPostgresStartIntent::replica(
        source.conninfo,
        managed_standby_auth_from_role_auth(
            &source.auth,
            runtime_config.postgres.paths.data_dir.as_path(),
        ),
        None,
    ))
}

fn materialize_start_postgres(
    runtime_config: &RuntimeConfig,
    intent_runtime: &super::state::ProcessRuntimePlan,
    mode: PostgresStartMode,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<ProcessExecutionKind, ProcessError> {
    let managed =
        materialize_managed_postgres_config(runtime_config, start_intent).map_err(|err| {
            ProcessError::InvalidSpec(format!("materialize managed postgres config failed: {err}"))
        })?;
    Ok(ProcessExecutionKind::StartPostgres(
        super::jobs::StartPostgresSpec {
            mode,
            data_dir: runtime_config.postgres.paths.data_dir.clone(),
            socket_dir: intent_runtime.postgres.paths.socket_dir.clone(),
            port: intent_runtime.postgres.port,
            config_file: managed.postgresql_conf_path,
            log_file: intent_runtime.postgres.paths.log_file.clone(),
            wait_seconds: None,
            timeout_ms: None,
        },
    ))
}

fn resolve_source_member<'a>(
    dcs: &'a DcsView,
    leader: &'a MemberId,
) -> Result<(&'a MemberId, &'a ClusterMemberView), ProcessError> {
    let cluster = dcs.cluster().ok_or_else(|| {
        ProcessError::InvalidSpec(
            "source member resolution requires a DCS cluster view, but DCS is currently not trusted"
                .to_string(),
        )
    })?;
    cluster.member(leader).map(|member| (leader, member)).ok_or_else(|| {
        ProcessError::InvalidSpec(format!(
            "target member `{}` not present in DCS view",
            leader.0
        ))
    })
}

fn source_materialization_error(error: super::source::SourceMaterializationError) -> ProcessError {
    ProcessError::InvalidSpec(error.to_string())
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), ProcessError> {
    if data_dir.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "wipe_data_dir data_dir must not be empty".to_string(),
        ));
    }
    if data_dir.exists() {
        wipe_data_dir_contents(data_dir)?;
    } else {
        fs::create_dir_all(data_dir).map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir create_dir_all failed: {err}"))
        })?;
    }
    set_postgres_data_dir_permissions(data_dir)?;
    Ok(())
}

fn wipe_data_dir_contents(data_dir: &Path) -> Result<(), ProcessError> {
    let entries = fs::read_dir(data_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!("wipe_data_dir read_dir failed: {err}"))
    })?;
    for entry_result in entries {
        let entry = entry_result.map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir read_dir entry failed: {err}"))
        })?;
        let file_type = entry.file_type().map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir file_type failed: {err}"))
        })?;
        let entry_path = entry.path();
        if file_type.is_dir() {
            fs::remove_dir_all(entry_path.as_path()).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "wipe_data_dir remove_dir_all failed for {}: {err}",
                    entry_path.display()
                ))
            })?;
        } else {
            fs::remove_file(entry_path.as_path()).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "wipe_data_dir remove_file failed for {}: {err}",
                    entry_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn set_postgres_data_dir_permissions(data_dir: &Path) -> Result<(), ProcessError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir set_permissions failed: {err}"))
        })?;
    }

    #[cfg(not(unix))]
    {
        let _path = data_dir;
    }

    Ok(())
}

fn binary_label(path: &std::path::Path) -> String {
    match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.trim().is_empty() => name.to_string(),
        _ => path.display().to_string(),
    }
}

fn validate_non_empty_path(field: &str, value: &std::path::Path) -> Result<(), ProcessError> {
    if value.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn validate_non_empty_pg_connect_target(
    field: &str,
    value: &crate::state::PgConnectTarget,
) -> Result<(), ProcessError> {
    let is_empty = match value {
        crate::state::PgConnectTarget::Tcp(target) => target.host().trim().is_empty(),
        crate::state::PgConnectTarget::Unix(target) => target.socket_dir.as_os_str().is_empty(),
    };
    if is_empty {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn render_pg_ctl_option_string(tokens: &[String]) -> Result<String, ProcessError> {
    let mut out = String::new();
    for (index, raw) in tokens.iter().enumerate() {
        let escaped = escape_pg_ctl_option_token(raw.as_str())?;
        if index > 0 {
            out.push(' ');
        }
        out.push_str(escaped.as_str());
    }
    Ok(out)
}

fn escape_pg_ctl_option_token(token: &str) -> Result<String, ProcessError> {
    if token.is_empty() {
        return Err(ProcessError::InvalidSpec(
            "pg_ctl option token must not be empty".to_string(),
        ));
    }
    if token.contains('\0') || token.contains('\n') || token.contains('\r') {
        return Err(ProcessError::InvalidSpec(
            "pg_ctl option token contains invalid characters".to_string(),
        ));
    }

    let needs_quotes = token.chars().any(|ch| ch.is_ascii_whitespace());
    if !needs_quotes {
        return Ok(token.to_string());
    }

    let mut out = String::with_capacity(token.len().saturating_add(2));
    out.push('"');
    for ch in token.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out.push('"');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Child, Command},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use tokio::sync::mpsc::unbounded_channel;

    use crate::{
        config::{HaConfig, ProcessTimeoutsConfig},
        dcs::DcsView,
        dev_support::runtime_config::{sample_binary_paths, RuntimeConfigBuilder},
        logging::LogSender,
        postgres_managed_conf::{managed_standby_passfile_path, MANAGED_POSTGRESQL_CONF_NAME},
        process::{
            jobs::{PostgresStartIntent, ProcessCommandRunner, ProcessCommandSpec, ProcessIntent},
            state::{
                ProcessCadence, ProcessControlPlane, ProcessIntentRequest, ProcessNodeIdentity,
                ProcessObservedState, ProcessRuntime, ProcessRuntimePlan, ProcessState,
                ProcessStateChannel, ProcessWorkerBootstrap, ProcessWorkerCtx,
            },
        },
        state::{new_state_channel, JobId, MemberId, StateSubscriber},
    };

    use super::start_job;
    use crate::process::postmaster::{lookup_managed_postmaster, ManagedPostmasterTarget};

    struct UnexpectedSpawnRunner;

    impl ProcessCommandRunner for UnexpectedSpawnRunner {
        fn spawn(
            &mut self,
            _spec: ProcessCommandSpec,
        ) -> Result<Box<dyn crate::process::jobs::ProcessHandle>, crate::process::jobs::ProcessError>
        {
            Err(crate::process::jobs::ProcessError::SpawnFailure {
                binary: "unexpected-spawn".to_string(),
                message: "spawn should not be called for start-postgres noop".to_string(),
            })
        }
    }

    struct ChildGuard(Option<Child>);

    impl ChildGuard {
        #[cfg(unix)]
        fn spawn_fake_postgres(
            root: &std::path::Path,
            data_dir: &std::path::Path,
        ) -> Result<Self, String> {
            let bin_dir = root.join("bin");
            fs::create_dir_all(&bin_dir).map_err(|err| {
                format!(
                    "create fake postgres bin dir {} failed: {err}",
                    bin_dir.display()
                )
            })?;
            let fake_postgres = bin_dir.join("postgres");
            fs::write(
                &fake_postgres,
                "#!/bin/bash\nexec -a postgres /bin/sleep 30\n",
            )
            .map_err(|err| {
                format!(
                    "write fake postgres script {} failed: {err}",
                    fake_postgres.display()
                )
            })?;
            let mut permissions = fs::metadata(&fake_postgres)
                .map_err(|err| {
                    format!(
                        "read fake postgres metadata {} failed: {err}",
                        fake_postgres.display()
                    )
                })?
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&fake_postgres, permissions).map_err(|err| {
                format!(
                    "set fake postgres script permissions {} failed: {err}",
                    fake_postgres.display()
                )
            })?;
            let child = Command::new(&fake_postgres)
                .arg(data_dir.display().to_string())
                .spawn()
                .map_err(|err| {
                    format!(
                        "spawn fake postgres process {} failed: {err}",
                        fake_postgres.display()
                    )
                })?;
            Ok(Self(Some(child)))
        }

        #[cfg(not(unix))]
        fn spawn_fake_postgres(
            _root: &std::path::Path,
            _data_dir: &std::path::Path,
        ) -> Result<Self, String> {
            Err("fake postgres helper is only implemented on unix".to_string())
        }
    }

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(child) = self.0.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-process-worker-{label}-{}-{millis}",
            std::process::id()
        ));
        fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    fn wait_for_fake_postgres_readiness(data_dir: &std::path::Path) -> Result<(), String> {
        let mut attempts = 0_u8;
        while attempts < 50 {
            let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
            if lookup_managed_postmaster(&target).is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "fake postgres readiness timed out for data_dir={}",
            data_dir.display()
        ))
    }

    fn build_test_ctx(
        data_dir: PathBuf,
        socket_dir: PathBuf,
        log_file: PathBuf,
    ) -> Result<(ProcessWorkerCtx, StateSubscriber<ProcessState>), String> {
        let cfg = RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir.clone())
            .transform_postgres(move |postgres| crate::config::PostgresConfig {
                paths: crate::config::PostgresPathsConfig {
                    data_dir: postgres.paths.data_dir.clone(),
                    socket_dir: Some(socket_dir.clone()),
                    log_file: Some(log_file.clone()),
                },
                ..postgres
            })
            .with_dcs_scope("cluster-a")
            .with_ha(HaConfig {
                loop_interval_ms: 500,
                lease_ttl_ms: 5_000,
            })
            .with_process(crate::config::ProcessConfig {
                timeouts: ProcessTimeoutsConfig {
                    pg_rewind_ms: 30_000,
                    bootstrap_ms: 30_000,
                    fencing_ms: 10_000,
                },
                working_root: std::path::PathBuf::from("/tmp/pgtuskmaster"),
                binaries: sample_binary_paths(),
            })
            .build();
        let initial = ProcessState::starting();
        let (publisher, subscriber) = new_state_channel(initial.clone());
        let (_cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
        let (_dcs_publisher, dcs_subscriber) = new_state_channel(DcsView::starting());
        let (_tx, inbox) = unbounded_channel();

        Ok((
            ProcessWorkerCtx::new(ProcessWorkerBootstrap {
                cadence: ProcessCadence {
                    poll_interval: Duration::from_millis(10),
                    now: Box::new(super::system_now_unix_millis),
                },
                config: cfg.process.clone(),
                identity: ProcessNodeIdentity {
                    self_id: MemberId(cfg.cluster.member_id.clone()),
                },
                observed: ProcessObservedState {
                    runtime_config,
                    dcs: dcs_subscriber,
                },
                plan: ProcessRuntimePlan::from_config(&cfg),
                state_channel: ProcessStateChannel {
                    current: initial,
                    publisher,
                    last_rejection: None,
                },
                control: ProcessControlPlane {
                    inbox,
                    inbox_disconnected_logged: false,
                    active_runtime: None,
                },
                runtime: ProcessRuntime {
                    log: LogSender::disabled(),
                    capture_subprocess_output: true,
                    command_runner: Box::new(UnexpectedSpawnRunner),
                },
            }),
            subscriber,
        ))
    }

    #[tokio::test]
    async fn start_postgres_noop_preserves_existing_standby_passfile() -> Result<(), String> {
        let root = unique_test_dir("noop-passfile")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let log_file = root.join("logs/postgres.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;

        let passfile_path = managed_standby_passfile_path(&data_dir);
        let original_passfile = "node-b:5432:replication:replicator:secret-password\n";
        fs::write(&passfile_path, original_passfile).map_err(|err| {
            format!(
                "write standby passfile {} failed: {err}",
                passfile_path.display()
            )
        })?;

        let fake_postgres = ChildGuard::spawn_fake_postgres(&root, &data_dir)?;
        let fake_postgres_pid = fake_postgres
            .0
            .as_ref()
            .map(std::process::Child::id)
            .ok_or_else(|| "fake postgres process handle missing child pid".to_string())?;
        let pid_contents = format!("{fake_postgres_pid}\n{}\n", data_dir.display());
        let pid_file = data_dir.join("postmaster.pid");
        fs::write(&pid_file, pid_contents)
            .map_err(|err| format!("write postmaster.pid {} failed: {err}", pid_file.display()))?;
        wait_for_fake_postgres_readiness(&data_dir)?;

        let _fake_postgres = fake_postgres;
        let (mut ctx, _state_subscriber) = build_test_ctx(data_dir.clone(), socket_dir, log_file)?;
        let request = ProcessIntentRequest {
            id: JobId("job-start-detached-standby-noop".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::DetachedStandby),
        };

        start_job(&mut ctx, request.clone())
            .await
            .map_err(|err| format!("start_job failed: {err}"))?;

        match &ctx.state_channel.current {
            ProcessState::Idle {
                last_outcome: Some(crate::process::state::JobOutcome::Success { id, job_kind, .. }),
                ..
            } => {
                if *id != request.id {
                    return Err(format!(
                        "unexpected job id after noop: expected={} actual={}",
                        request.id.0, id.0
                    ));
                }
                if *job_kind != crate::process::jobs::ActiveJobKind::StartDetachedStandby {
                    return Err(format!(
                        "unexpected job kind after noop: expected={:?} actual={job_kind:?}",
                        crate::process::jobs::ActiveJobKind::StartDetachedStandby
                    ));
                }
            }
            other => {
                return Err(format!(
                    "expected idle success after start noop, observed {other:?}"
                ));
            }
        }

        let preserved = fs::read_to_string(&passfile_path).map_err(|err| {
            format!(
                "read standby passfile {} failed: {err}",
                passfile_path.display()
            )
        })?;
        if preserved != original_passfile {
            return Err(format!(
                "standby passfile changed during noop: expected={original_passfile:?} actual={preserved:?}"
            ));
        }

        let managed_conf = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
        if managed_conf.exists() {
            return Err(format!(
                "managed postgres conf should not be materialized for noop start at {}",
                managed_conf.display()
            ));
        }

        Ok(())
    }
}


===== src/process/log_event.rs =====
use std::borrow::Cow;

use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    LogParser, LogProducer, LogTransport, SealedLogEvent, SeverityText,
};

use super::jobs::ProcessJobKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessLogOrigin {
    Run,
    StepOnce,
    StartJob,
    TickActiveJob,
    EmitSubprocessLine,
}

impl ProcessLogOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Run => "process_worker::run",
            Self::StepOnce => "process_worker::step_once",
            Self::StartJob => "process_worker::start_job",
            Self::TickActiveJob => "process_worker::tick_active_job",
            Self::EmitSubprocessLine => "process_worker::emit_subprocess_line",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobIdentity {
    pub(crate) job_id: String,
    pub(crate) kind: ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessExecutionIdentity {
    pub(crate) job: ProcessJobIdentity,
    pub(crate) binary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CapturedStream {
    Stdout,
    Stderr,
}

impl CapturedStream {
    fn severity(self) -> SeverityText {
        match self {
            Self::Stdout => SeverityText::Info,
            Self::Stderr => SeverityText::Warn,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }

    fn transport(self) -> LogTransport {
        match self {
            Self::Stdout => LogTransport::ChildStdout,
            Self::Stderr => LogTransport::ChildStderr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessLogEvent {
    WorkerRunStarted {
        origin: ProcessLogOrigin,
        capture_subprocess_output: bool,
    },
    RequestReceived {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
    },
    InboxDisconnected {
        origin: ProcessLogOrigin,
    },
    BusyRejected {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
    },
    StartPostgresAlreadyRunning {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        data_dir: String,
    },
    StartPostgresPreflightFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    IntentMaterializationFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    BuildCommandFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    SpawnFailed {
        origin: ProcessLogOrigin,
        job: ProcessJobIdentity,
        error: String,
    },
    Started {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
    },
    OutputDrainFailed {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        error: String,
    },
    Timeout {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
    },
    ExitedSuccessfully {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
    },
    ExitedUnsuccessfully {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        error: String,
    },
    PollFailed {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        error: String,
    },
    OutputEmitFailed {
        origin: ProcessLogOrigin,
        execution: ProcessExecutionIdentity,
        stream: CapturedStream,
        bytes_len: usize,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubprocessLogEvent {
    pub(crate) producer: LogProducer,
    pub(crate) origin: ProcessLogOrigin,
    pub(crate) execution: ProcessExecutionIdentity,
    pub(crate) stream: CapturedStream,
    pub(crate) bytes: Vec<u8>,
}

impl SealedLogEvent for ProcessLogEvent {}

impl DomainLogEvent for ProcessLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::WorkerRunStarted { origin, .. } => event_metadata(
                SeverityText::Debug,
                "process worker run started",
                "process.worker.run_started",
                LogEventResult::Ok,
                *origin,
            ),
            Self::RequestReceived { origin, .. } => event_metadata(
                SeverityText::Debug,
                "process job request received",
                "process.worker.request_received",
                LogEventResult::Ok,
                *origin,
            ),
            Self::InboxDisconnected { origin } => event_metadata(
                SeverityText::Warn,
                "process worker inbox disconnected",
                "process.worker.inbox_disconnected",
                LogEventResult::Failed,
                *origin,
            ),
            Self::BusyRejected { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process worker busy; rejecting job",
                "process.worker.busy_reject",
                LogEventResult::Failed,
                *origin,
            ),
            Self::StartPostgresAlreadyRunning { origin, .. } => event_metadata(
                SeverityText::Info,
                "start postgres preflight: postgres already running",
                "process.job.start_postgres_noop",
                LogEventResult::Ok,
                *origin,
            ),
            Self::StartPostgresPreflightFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "start postgres preflight failed",
                "process.job.start_postgres_preflight_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::IntentMaterializationFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process intent materialization failed",
                "process.worker.intent_materialization_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::BuildCommandFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process build command failed",
                "process.job.build_command_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::SpawnFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process spawn failed",
                "process.job.spawn_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::Started { origin, .. } => event_metadata(
                SeverityText::Info,
                "process job started",
                "process.job.started",
                LogEventResult::Ok,
                *origin,
            ),
            Self::OutputDrainFailed { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process output drain failed",
                "process.worker.output_drain_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::Timeout { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process job timed out; cancelling",
                "process.job.timeout",
                LogEventResult::Timeout,
                *origin,
            ),
            Self::ExitedSuccessfully { origin, .. } => event_metadata(
                SeverityText::Info,
                "process job exited successfully",
                "process.job.exited",
                LogEventResult::Ok,
                *origin,
            ),
            Self::ExitedUnsuccessfully { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process job exited unsuccessfully",
                "process.job.exited",
                LogEventResult::Failed,
                *origin,
            ),
            Self::PollFailed { origin, .. } => event_metadata(
                SeverityText::Error,
                "process job poll failed",
                "process.job.poll_failed",
                LogEventResult::Failed,
                *origin,
            ),
            Self::OutputEmitFailed { origin, .. } => event_metadata(
                SeverityText::Warn,
                "process subprocess output emit failed",
                "process.worker.output_emit_failed",
                LogEventResult::Failed,
                *origin,
            ),
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::WorkerRunStarted {
                capture_subprocess_output,
                ..
            } => visitor.bool("capture_subprocess_output", *capture_subprocess_output),
            Self::RequestReceived { job, .. } | Self::BusyRejected { job, .. } => {
                write_job(visitor, job);
            }
            Self::InboxDisconnected { .. } => {}
            Self::StartPostgresAlreadyRunning { job, data_dir, .. } => {
                write_job(visitor, job);
                visitor.string("data_dir", data_dir.clone());
            }
            Self::StartPostgresPreflightFailed { job, error, .. }
            | Self::IntentMaterializationFailed { job, error, .. }
            | Self::BuildCommandFailed { job, error, .. }
            | Self::SpawnFailed { job, error, .. } => {
                write_job(visitor, job);
                visitor.string("error", error.clone());
            }
            Self::Started { execution, .. }
            | Self::Timeout { execution, .. }
            | Self::ExitedSuccessfully { execution, .. } => {
                write_execution(visitor, execution);
            }
            Self::OutputDrainFailed {
                execution,
                error,
                ..
            }
            | Self::ExitedUnsuccessfully {
                execution,
                error,
                ..
            }
            | Self::PollFailed {
                execution,
                error,
                ..
            } => {
                write_execution(visitor, execution);
                visitor.string("error", error.clone());
            }
            Self::OutputEmitFailed {
                execution,
                stream,
                bytes_len,
                error,
                ..
            } => {
                write_execution(visitor, execution);
                visitor.str("stream", stream.label());
                visitor.usize("bytes_len", *bytes_len);
                visitor.string("error", error.clone());
            }
        }
    }
}

impl SealedLogEvent for SubprocessLogEvent {}

impl DomainLogEvent for SubprocessLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        LogEventMetadata {
            severity: self.stream.severity(),
            message: Cow::Owned(String::from_utf8_lossy(self.bytes.as_slice()).into_owned()),
            event_name: "process.subprocess.line",
            event_domain: "process",
            event_result: LogEventResult::Ok,
            source: LogEventSource::new(
                self.producer,
                self.stream.transport(),
                LogParser::Raw,
                self.origin.label(),
            ),
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        write_execution(visitor, &self.execution);
        visitor.str("stream", self.stream.label());
    }
}

fn event_metadata(
    severity: SeverityText,
    message: &'static str,
    event_name: &'static str,
    event_result: LogEventResult,
    origin: ProcessLogOrigin,
) -> LogEventMetadata {
    LogEventMetadata {
        severity,
        message: Cow::Borrowed(message),
        event_name,
        event_domain: "process",
        event_result,
        source: LogEventSource::app(origin.label()),
    }
}

fn write_job(visitor: &mut dyn LogFieldVisitor, identity: &ProcessJobIdentity) {
    visitor.string("job.id", identity.job_id.clone());
    visitor.str("job.kind", process_job_kind_label(identity.kind));
}

fn write_execution(visitor: &mut dyn LogFieldVisitor, execution: &ProcessExecutionIdentity) {
    write_job(visitor, &execution.job);
    visitor.string("binary", execution.binary.clone());
}

fn process_job_kind_label(kind: ProcessJobKind) -> &'static str {
    match kind {
        ProcessJobKind::Bootstrap => "bootstrap",
        ProcessJobKind::BaseBackup => "basebackup",
        ProcessJobKind::PgRewind => "pg_rewind",
        ProcessJobKind::Promote => "promote",
        ProcessJobKind::Demote => "demote",
        ProcessJobKind::StartPostgres => "start_postgres",
        ProcessJobKind::StartPrimary => "start_primary",
        ProcessJobKind::StartDetachedStandby => "start_detached_standby",
        ProcessJobKind::StartReplica => "start_replica",
    }
}


===== src/logging/mod.rs =====
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{dispatcher, Dispatch};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

mod event;
mod raw_record;

pub(crate) mod postgres_ingest;
pub(crate) mod tailer;

pub(crate) use event::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    SealedLogEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SeverityText {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl SeverityText {
    pub(crate) fn number(self) -> u8 {
        // OpenTelemetry severity_number mapping.
        match self {
            Self::Trace => 1,
            Self::Debug => 5,
            Self::Info => 9,
            Self::Warn => 13,
            Self::Error => 17,
            Self::Fatal => 21,
        }
    }
}

impl From<crate::config::LogLevel> for SeverityText {
    fn from(value: crate::config::LogLevel) -> Self {
        match value {
            crate::config::LogLevel::Trace => Self::Trace,
            crate::config::LogLevel::Debug => Self::Debug,
            crate::config::LogLevel::Info => Self::Info,
            crate::config::LogLevel::Warn => Self::Warn,
            crate::config::LogLevel::Error => Self::Error,
            crate::config::LogLevel::Fatal => Self::Fatal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogProducer {
    App,
    Postgres,
    PgTool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogTransport {
    Internal,
    FileTail,
    ChildStdout,
    ChildStderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogParser {
    App,
    PostgresJson,
    PostgresPlain,
    Raw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LogSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) parser: LogParser,
    pub(crate) origin: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct LogRecord {
    pub(crate) timestamp_ms: u64,
    pub(crate) hostname: String,
    pub(crate) severity_text: SeverityText,
    pub(crate) severity_number: u8,
    pub(crate) message: String,
    pub(crate) source: LogSource,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) attributes: BTreeMap<String, Value>,
}

impl LogRecord {
    #[cfg(test)]
    pub(crate) fn new(
        timestamp_ms: u64,
        hostname: String,
        severity_text: SeverityText,
        message: String,
        source: LogSource,
    ) -> Self {
        Self {
            timestamp_ms,
            hostname,
            severity_text,
            severity_number: severity_text.number(),
            message,
            source,
            attributes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum LogError {
    #[error("json serialize failed: {0}")]
    Json(String),
    #[error("sink write failed: {0}")]
    SinkIo(String),
}

#[derive(Debug, Error)]
pub(crate) enum LogBootstrapError {
    #[error("logging misconfigured: {0}")]
    Misconfigured(String),
    #[error("sink init failed: {0}")]
    SinkInit(String),
}

pub(crate) trait LogSink: Send + Sync {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError>;
}

pub(crate) struct JsonlStderrSink {
    stderr: Mutex<std::io::Stderr>,
}

impl JsonlStderrSink {
    pub(crate) fn new() -> Self {
        Self {
            stderr: Mutex::new(std::io::stderr()),
        }
    }
}

impl LogSink for JsonlStderrSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut stderr = self
            .stderr
            .lock()
            .map_err(|_| LogError::SinkIo("stderr lock poisoned".to_string()))?;
        stderr
            .write_all(line.as_bytes())
            .and_then(|()| stderr.write_all(b"\n"))
            .map_err(|err| LogError::SinkIo(err.to_string()))?;
        Ok(())
    }
}

struct NullSink;

impl LogSink for NullSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _ = record;
        Ok(())
    }
}

pub(crate) struct JsonlFileSink {
    path: PathBuf,
    writer: Mutex<LineWriter<File>>,
}

impl JsonlFileSink {
    pub(crate) fn new(path: PathBuf, mode: crate::config::FileSinkMode) -> Result<Self, LogError> {
        if path.as_os_str().is_empty() {
            return Err(LogError::SinkIo("file sink path is empty".to_string()));
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    LogError::SinkIo(format!(
                        "create log directory {} for {} failed: {err}",
                        parent.display(),
                        path.display()
                    ))
                })?;
            }
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        match mode {
            crate::config::FileSinkMode::Append => {
                options.append(true);
            }
            crate::config::FileSinkMode::Truncate => {
                options.truncate(true);
            }
        }

        let file = options.open(&path).map_err(|err| {
            LogError::SinkIo(format!("open log file {} failed: {err}", path.display()))
        })?;

        Ok(Self {
            path,
            writer: Mutex::new(LineWriter::new(file)),
        })
    }
}

impl LogSink for JsonlFileSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| LogError::SinkIo("file sink lock poisoned".to_string()))?;
        writer
            .write_all(line.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .map_err(|err| {
                LogError::SinkIo(format!(
                    "write to log file {} failed: {err}",
                    self.path.display()
                ))
            })?;
        Ok(())
    }
}

struct FanoutSink {
    sinks: Vec<(String, Arc<dyn LogSink>)>,
}

static FANOUT_DIAGNOSTIC_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
static FANOUT_DIAGNOSTIC_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

struct AtomicResetGuard<'a> {
    value: &'a AtomicBool,
}

impl Drop for AtomicResetGuard<'_> {
    fn drop(&mut self) {
        self.value.store(false, Ordering::SeqCst);
    }
}

impl FanoutSink {
    fn new(sinks: Vec<(String, Arc<dyn LogSink>)>) -> Self {
        Self { sinks }
    }

    fn write_diagnostic(label: &str, err: &LogError) {
        let acquired = FANOUT_DIAGNOSTIC_ACTIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if !acquired {
            return;
        }

        let _guard = AtomicResetGuard {
            value: &FANOUT_DIAGNOSTIC_ACTIVE,
        };

        #[cfg(test)]
        {
            FANOUT_DIAGNOSTIC_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(b"fanout sink failure: ");
        let _ = stderr.write_all(label.as_bytes());
        let _ = stderr.write_all(b": ");
        let _ = stderr.write_all(err.to_string().as_bytes());
        let _ = stderr.write_all(b"\n");
    }
}

impl LogSink for FanoutSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut ok_count: u64 = 0;
        let mut failures: Vec<(String, String)> = Vec::new();

        for (label, sink) in &self.sinks {
            match sink.emit(record) {
                Ok(()) => {
                    ok_count += 1;
                }
                Err(err) => {
                    Self::write_diagnostic(label.as_str(), &err);
                    failures.push((label.clone(), err.to_string()));
                }
            }
        }

        if ok_count > 0 {
            return Ok(());
        }

        let mut message = "all sinks failed".to_string();
        if !failures.is_empty() {
            message.push_str(": ");
            for (idx, (label, err)) in failures.iter().enumerate() {
                if idx > 0 {
                    message.push_str("; ");
                }
                message.push_str(label.as_str());
                message.push_str(" => ");
                message.push_str(err.as_str());
            }
        }
        Err(LogError::SinkIo(message))
    }
}

const TRACING_LOG_TARGET: &str = "pgtuskmaster::logging::record";

thread_local! {
    static CURRENT_TRACING_RECORD: RefCell<Option<LogRecord>> = const { RefCell::new(None) };
    static CURRENT_TRACING_RESULT: RefCell<Option<Result<(), LogError>>> = const { RefCell::new(None) };
}

struct ActiveTracingRecordGuard;

impl ActiveTracingRecordGuard {
    fn new(record: &LogRecord) -> Result<Self, LogError> {
        CURRENT_TRACING_RECORD.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err(LogError::SinkIo(
                    "nested tracing-backed log emission is not supported".to_string(),
                ));
            }
            *slot = Some(record.clone());
            Ok(())
        })?;
        CURRENT_TRACING_RESULT.with(|slot| {
            *slot.borrow_mut() = None;
        });
        Ok(Self)
    }
}

impl Drop for ActiveTracingRecordGuard {
    fn drop(&mut self) {
        CURRENT_TRACING_RECORD.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
        CURRENT_TRACING_RESULT.with(|slot| {
            let _ = slot.borrow_mut().take();
        });
    }
}

struct TracingRecordLayer {
    sink: Arc<dyn LogSink>,
}

impl<S> Layer<S> for TracingRecordLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != TRACING_LOG_TARGET {
            return;
        }

        let result = CURRENT_TRACING_RECORD.with(|slot| {
            let slot = slot.borrow();
            match slot.as_ref() {
                Some(record) => self.sink.emit(record),
                None => Err(LogError::SinkIo(
                    "tracing backend event emitted without an active record".to_string(),
                )),
            }
        });

        CURRENT_TRACING_RESULT.with(|slot| {
            *slot.borrow_mut() = Some(result);
        });
    }
}

#[derive(Clone)]
struct TracingBackend {
    dispatch: Dispatch,
}

impl TracingBackend {
    fn new(sink: Arc<dyn LogSink>) -> Self {
        let subscriber = Registry::default().with(TracingRecordLayer { sink });
        Self {
            dispatch: Dispatch::new(subscriber),
        }
    }

    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _guard = ActiveTracingRecordGuard::new(record)?;
        dispatcher::with_default(&self.dispatch, || dispatch_tracing_record_event(record));
        CURRENT_TRACING_RESULT.with(|slot| {
            slot.borrow_mut().take().unwrap_or_else(|| {
                Err(LogError::SinkIo(
                    "tracing backend did not produce an emission result".to_string(),
                ))
            })
        })
    }
}

fn dispatch_tracing_record_event(record: &LogRecord) {
    match record.severity_text {
        SeverityText::Trace => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::TRACE,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Debug => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::DEBUG,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Info => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::INFO,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Warn => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::WARN,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
        SeverityText::Error | SeverityText::Fatal => tracing::event!(
            target: TRACING_LOG_TARGET,
            tracing::Level::ERROR,
            origin = record.source.origin.as_str(),
            producer = ?record.source.producer,
            transport = ?record.source.transport,
            parser = ?record.source.parser,
            severity_number = record.severity_number,
            message = record.message.as_str()
        ),
    }
}

#[derive(Clone)]
enum LogSenderMode {
    Disabled,
    Queue(mpsc::UnboundedSender<raw_record::QueuedRecord>),
}

#[derive(Clone)]
pub(crate) struct LogSender {
    hostname: String,
    mode: LogSenderMode,
    min_app_severity_number: u8,
}

impl std::fmt::Debug for LogSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogSender")
            .field("hostname", &self.hostname)
            .field("min_app_severity_number", &self.min_app_severity_number)
            .finish()
    }
}

#[derive(Debug, Error)]
pub(crate) enum LogSendError {
    #[error("log queue is closed")]
    QueueClosed,
}

pub(crate) struct LogWorker {
    receiver: mpsc::UnboundedReceiver<raw_record::QueuedRecord>,
    backend: Arc<TracingBackend>,
}

impl LogWorker {
    pub(crate) async fn run(mut self) {
        while let Some(record) = self.receiver.recv().await {
            let materialized = record.into_record();
            let _ = self.backend.emit(&materialized);
        }
    }
}

impl LogSender {
    pub(crate) fn new(
        hostname: String,
        sender: mpsc::UnboundedSender<raw_record::QueuedRecord>,
        min_app_severity: SeverityText,
    ) -> Self {
        Self {
            hostname,
            mode: LogSenderMode::Queue(sender),
            min_app_severity_number: min_app_severity.number(),
        }
    }

    pub(crate) fn disabled() -> Self {
        Self {
            hostname: "unknown".to_string(),
            mode: LogSenderMode::Disabled,
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    pub(crate) fn send<E>(&self, event: E) -> Result<(), LogSendError>
    where
        E: DomainLogEvent,
    {
        if event.metadata().severity.number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = raw_record::QueuedRecord::from_event(
            system_now_unix_millis(),
            self.hostname.clone(),
            event,
        );
        match &self.mode {
            LogSenderMode::Disabled => Ok(()),
            LogSenderMode::Queue(sender) => sender.send(record).map_err(|_| LogSendError::QueueClosed),
        }
    }
}

pub(crate) fn system_now_unix_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

fn detect_hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "unknown".to_string(),
    }
}

pub(crate) struct LoggingSystem {
    pub(crate) sender: LogSender,
    pub(crate) worker: LogWorker,
}

pub(crate) fn bootstrap(
    cfg: &crate::config::RuntimeConfig,
) -> Result<LoggingSystem, LogBootstrapError> {
    let hostname = detect_hostname();
    let mut sinks: Vec<(String, Arc<dyn LogSink>)> = Vec::new();

    if cfg.logging.sinks.stderr.enabled {
        sinks.push((
            "stderr".to_string(),
            Arc::new(JsonlStderrSink::new()) as Arc<dyn LogSink>,
        ));
    }

    if cfg.logging.sinks.file.enabled {
        let path = cfg.logging.sinks.file.path.clone().ok_or_else(|| {
            LogBootstrapError::Misconfigured(
                "logging.sinks.file.enabled=true but logging.sinks.file.path is not set"
                    .to_string(),
            )
        })?;

        let label = format!("file:{}", path.display());
        let sink = JsonlFileSink::new(path, cfg.logging.sinks.file.mode)
            .map_err(|err| LogBootstrapError::SinkInit(err.to_string()))?;
        sinks.push((label, Arc::new(sink) as Arc<dyn LogSink>));
    }

    let sink: Arc<dyn LogSink> = match sinks.len() {
        0 => Arc::new(NullSink),
        1 => sinks
            .pop()
            .map(|(_label, sink)| sink)
            .ok_or_else(|| LogBootstrapError::SinkInit("unexpected empty sink list".to_string()))?,
        _ => Arc::new(FanoutSink::new(sinks)),
    };

    let backend = Arc::new(TracingBackend::new(sink));
    let (sender, receiver) = mpsc::unbounded_channel();

    Ok(LoggingSystem {
        sender: LogSender::new(hostname, sender, SeverityText::from(cfg.logging.level)),
        worker: LogWorker { receiver, backend },
    })
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct TestSink {
    records: Arc<Mutex<Vec<LogRecord>>>,
}

#[cfg(test)]
impl TestSink {
    pub(crate) fn take(&self) -> Vec<LogRecord> {
        let mut locked = match self.records.lock() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *locked)
    }
}

#[cfg(test)]
impl LogSink for TestSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut locked = self
            .records
            .lock()
            .map_err(|_| LogError::SinkIo("test sink lock poisoned".to_string()))?;
        locked.push(record.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::{
        DebugConfig, LogCleanupConfig, LogLevel, LoggingConfig, PostgresLoggingConfig,
        RuntimeConfig,
    };
    use crate::process::jobs::ProcessJobKind;
    use crate::process::log_event::{
        CapturedStream, ProcessExecutionIdentity, ProcessJobIdentity, ProcessLogOrigin,
        SubprocessLogEvent,
    };
    use crate::runtime::log_event::{RuntimeLogEvent, RuntimeLogOrigin, RuntimeNodeIdentity};

    fn unique_temp_root(label: &str) -> PathBuf {
        let pid = std::process::id();
        static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::env::temp_dir().join(format!("pgtuskmaster-{label}-{pid}-{unique}"))
    }

    fn remove_dir_all_if_exists(path: &std::path::Path) -> Result<(), std::io::Error> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn remove_file_if_exists(path: &std::path::Path) -> Result<(), std::io::Error> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn sample_record(message: &str) -> LogRecord {
        LogRecord::new(
            1,
            "host-a".to_string(),
            SeverityText::Info,
            message.to_string(),
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )
    }

    fn read_lines(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        Ok(text
            .lines()
            .map(|line| line.to_string())
            .filter(|line| !line.trim().is_empty())
            .collect())
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::dev_support::runtime_config::RuntimeConfigBuilder::new()
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..crate::dev_support::runtime_config::sample_postgres_logging_config()
                            .cleanup
                    },
                    ..crate::dev_support::runtime_config::sample_postgres_logging_config()
                },
                ..crate::dev_support::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    fn sample_runtime_event() -> RuntimeLogEvent {
        RuntimeLogEvent::StartupEntered {
            origin: RuntimeLogOrigin::RunNodeFromConfig,
            identity: RuntimeNodeIdentity {
                scope: "scope-a".to_string(),
                member_id: "member-a".to_string(),
            },
            startup_run_id: "run-1".to_string(),
            logging_level: crate::config::LogLevel::Info,
        }
    }

    fn test_log_system(min_app_severity: SeverityText) -> (LogSender, LogWorker, TestSink) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sink = TestSink::default();
        let sink_dyn: Arc<dyn LogSink> = Arc::new(sink.clone());
        (
            LogSender::new("host-a".to_string(), sender, min_app_severity),
            LogWorker {
                receiver,
                backend: Arc::new(TracingBackend::new(sink_dyn)),
            },
            sink,
        )
    }

    fn run_worker(worker: LogWorker) -> Result<(), Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(worker.run());
        Ok(())
    }

    fn collect_records<E>(
        min_app_severity: SeverityText,
        event: E,
    ) -> Result<Vec<LogRecord>, Box<dyn std::error::Error>>
    where
        E: DomainLogEvent,
    {
        let (log, worker, sink) = test_log_system(min_app_severity);
        log.send(event)?;
        drop(log);
        run_worker(worker)?;
        Ok(sink.take())
    }

    #[test]
    fn emit_typed_runtime_event_encodes_headers_and_fields(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let records = collect_records(SeverityText::Trace, sample_runtime_event())?;
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].attributes.get("event.name"),
            Some(&Value::String("runtime.startup.entered".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("event.domain"),
            Some(&Value::String("runtime".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("event.result"),
            Some(&Value::String("ok".to_string()))
        );
        assert_eq!(records[0].source.origin, "runtime::run_node_from_config");
        assert_eq!(records[0].message, "runtime starting");
        assert_eq!(
            records[0].attributes.get("scope"),
            Some(&Value::String("scope-a".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("member_id"),
            Some(&Value::String("member-a".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("startup_run_id"),
            Some(&Value::String("run-1".to_string()))
        );
        assert_eq!(
            records[0].attributes.get("logging.level"),
            Some(&Value::String("info".to_string()))
        );
        Ok(())
    }

    #[test]
    fn emit_typed_event_respects_min_severity() -> Result<(), Box<dyn std::error::Error>> {
        let records = collect_records(SeverityText::Warn, sample_runtime_event())?;
        assert!(records.is_empty());
        Ok(())
    }

    #[test]
    fn subprocess_line_event_encodes_stream_metadata(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let records = collect_records(
            SeverityText::Trace,
            SubprocessLogEvent {
            producer: LogProducer::PgTool,
            origin: ProcessLogOrigin::EmitSubprocessLine,
            execution: ProcessExecutionIdentity {
                job: ProcessJobIdentity {
                    job_id: "job-1".to_string(),
                    kind: ProcessJobKind::StartPostgres,
                },
                binary: "postgres".to_string(),
            },
            stream: CapturedStream::Stderr,
            bytes: vec![0xff_u8, 0x00, b'a', 0x80],
        },
        )?;

        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.source.producer, LogProducer::PgTool);
        assert_eq!(record.source.transport, LogTransport::ChildStderr);
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.source.origin, "process_worker::emit_subprocess_line");
        assert_eq!(record.severity_text, SeverityText::Warn);
        assert!(record.message.contains('a'));
        assert_eq!(
            record.attributes.get("job.id"),
            Some(&Value::String("job-1".to_string()))
        );
        assert_eq!(
            record.attributes.get("job.kind"),
            Some(&Value::String("start_postgres".to_string()))
        );
        assert_eq!(
            record.attributes.get("stream"),
            Some(&Value::String("stderr".to_string()))
        );
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_creates_parent_dirs_and_writes_jsonl_line(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-create");
        remove_dir_all_if_exists(&root)?;

        let path = root.join("a").join("b").join("log.jsonl");
        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("hello"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_append_mode_preserves_existing_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-append");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"pre\":1}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("post"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 2);
        let pre: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(pre["pre"], 1);
        let post: serde_json::Value = serde_json::from_str(lines[1].as_str())?;
        assert_eq!(post["message"], "post");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_truncate_mode_replaces_existing_content(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-truncate");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"stale\":true}\n{\"stale\":true}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Truncate)?;
        sink.emit(&sample_record("fresh"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fresh");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_errors_when_parent_is_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-parent-file");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let not_a_dir = root.join("not_a_dir");
        remove_file_if_exists(&not_a_dir)?;
        let write_res = std::fs::write(&not_a_dir, b"im a file");
        assert!(write_res.is_ok());

        let path = not_a_dir.join("app.jsonl");
        let err = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append);
        assert!(matches!(err, Err(LogError::SinkIo(_))));

        if let Err(LogError::SinkIo(msg)) = err {
            assert!(msg.contains(path.display().to_string().as_str()));
        }

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[derive(Clone)]
    struct FailSink;

    impl LogSink for FailSink {
        fn emit(&self, _record: &LogRecord) -> Result<(), LogError> {
            Err(LogError::SinkIo("fail sink".to_string()))
        }
    }

    #[test]
    fn fanout_sink_ok_when_any_sink_succeeds_and_emits_diagnostic() {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = Arc::new(TestSink::default());
        let ok_dyn: Arc<dyn LogSink> = ok;
        let fail_dyn: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![
            ("ok".to_string(), ok_dyn),
            ("fail".to_string(), fail_dyn),
        ]);

        assert!(sink.emit(&sample_record("x")).is_ok());
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn fanout_sink_err_when_all_sinks_fail() {
        let fail_a: Arc<dyn LogSink> = Arc::new(FailSink);
        let fail_b: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![("a".to_string(), fail_a), ("b".to_string(), fail_b)]);

        let err = sink.emit(&sample_record("x"));
        assert!(matches!(err, Err(LogError::SinkIo(_))));
    }

    #[test]
    fn sender_reports_only_queue_closed_to_callers() {
        let (sender, receiver) = mpsc::unbounded_channel();
        drop(receiver);
        let log = LogSender::new("host-a".to_string(), sender, SeverityText::Trace);

        let err = log.send(sample_runtime_event());
        assert!(matches!(err, Err(LogSendError::QueueClosed)));
    }

    #[test]
    fn worker_keeps_sink_failures_internal_after_enqueue(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let sink: Arc<dyn LogSink> = Arc::new(FailSink);
        let worker = LogWorker {
            receiver,
            backend: Arc::new(TracingBackend::new(sink)),
        };
        let log = LogSender::new("host-a".to_string(), sender, SeverityText::Trace);

        assert!(log.send(sample_runtime_event()).is_ok());
        drop(log);
        run_worker(worker)?;
        Ok(())
    }

    #[test]
    fn worker_preserves_partial_fanout_success() -> Result<(), Box<dyn std::error::Error>> {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = TestSink::default();
        let ok_records = ok.clone();
        let sink: Arc<dyn LogSink> = Arc::new(FanoutSink::new(vec![
            ("ok".to_string(), Arc::new(ok) as Arc<dyn LogSink>),
            ("fail".to_string(), Arc::new(FailSink) as Arc<dyn LogSink>),
        ]));
        let (sender, receiver) = mpsc::unbounded_channel();
        let worker = LogWorker {
            receiver,
            backend: Arc::new(TracingBackend::new(sink)),
        };
        let log = LogSender::new("host-a".to_string(), sender, SeverityText::Trace);

        log.send(sample_runtime_event())?;
        drop(log);
        run_worker(worker)?;

        let records = ok_records.take();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "runtime starting");
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
        Ok(())
    }

    #[test]
    fn bootstrap_file_enabled_without_path_returns_misconfigured() {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let res = bootstrap(&cfg);
        assert!(matches!(res, Err(LogBootstrapError::Misconfigured(_))));
    }

    #[test]
    fn bootstrap_file_enabled_with_path_writes_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("bootstrap-file-enabled");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let LoggingSystem { sender, worker } = bootstrap(&cfg)?;
        sender.send(sample_runtime_event())?;
        drop(sender);
        run_worker(worker)?;

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "runtime starting");
        assert_eq!(v["severity_text"], "info");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_stderr_and_file_still_writes_file() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = unique_temp_root("bootstrap-stderr-and-file");
        remove_dir_all_if_exists(&root)?;
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = true;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let LoggingSystem { sender, worker } = bootstrap(&cfg)?;
        sender.send(sample_runtime_event())?;
        drop(sender);
        run_worker(worker)?;

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "runtime starting");

        remove_dir_all_if_exists(&root)?;
        Ok(())
    }

    #[test]
    fn bootstrap_with_all_sinks_disabled_is_non_fatal() -> Result<(), LogBootstrapError> {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = false;

        let system = bootstrap(&cfg)?;
        let res = system.sender.send(sample_runtime_event());
        assert!(res.is_ok(), "expected null sink to accept record: {res:?}");
        Ok(())
    }
}


===== src/logging/postgres_ingest.rs =====
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

use std::borrow::Cow;

use serde_json::Value;

use crate::config::{LogCleanupConfig, RuntimeConfig};
use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    LogProducer, LogSender, LogTransport, SealedLogEvent, SeverityText,
};
use crate::state::WorkerError;

use super::tailer::{DirTailers, FileTailer, StartPosition};

pub(crate) struct PostgresIngestWorkerCtx {
    pub(crate) cfg: RuntimeConfig,
    pub(crate) log: LogSender,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PostgresIngestOrigin {
    Run,
}

impl PostgresIngestOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::Run => "postgres_ingest::run",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PostgresIngestLogEvent {
    StepOnceFailed {
        origin: PostgresIngestOrigin,
        attempts: u32,
        suppressed: u64,
        error: String,
    },
    Recovered {
        origin: PostgresIngestOrigin,
        attempts: u32,
    },
    IterationSummary {
        origin: PostgresIngestOrigin,
        pg_ctl_lines_emitted: u64,
        log_dir_files_tailed: u64,
        log_dir_lines_emitted: u64,
        dir_tailers: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PostgresLineSource {
    producer: LogProducer,
    transport: LogTransport,
    origin: String,
    path: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
enum PostgresLineLogEvent {
    Json {
        source: PostgresLineSource,
        severity: SeverityText,
        message: String,
        payload: Value,
    },
    Plain {
        source: PostgresLineSource,
        severity: SeverityText,
        message: String,
        level_raw: String,
    },
    Unparsed {
        source: PostgresLineSource,
        decoded_line: String,
    },
}

impl SealedLogEvent for PostgresIngestLogEvent {}

impl DomainLogEvent for PostgresIngestLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::StepOnceFailed { origin, .. } => LogEventMetadata {
                severity: SeverityText::Error,
                message: Cow::Borrowed("postgres ingest step once failed"),
                event_name: "postgres_ingest.step_once_failed",
                event_domain: "postgres_ingest",
                event_result: LogEventResult::Failed,
                source: LogEventSource::app(origin.label()),
            },
            Self::Recovered { origin, .. } => LogEventMetadata {
                severity: SeverityText::Info,
                message: Cow::Borrowed("postgres ingest recovered"),
                event_name: "postgres_ingest.recovered",
                event_domain: "postgres_ingest",
                event_result: LogEventResult::Recovered,
                source: LogEventSource::app(origin.label()),
            },
            Self::IterationSummary { origin, .. } => LogEventMetadata {
                severity: SeverityText::Debug,
                message: Cow::Borrowed("postgres ingest iteration summary"),
                event_name: "postgres_ingest.iteration_summary",
                event_domain: "postgres_ingest",
                event_result: LogEventResult::Ok,
                source: LogEventSource::app(origin.label()),
            },
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::StepOnceFailed {
                attempts,
                suppressed,
                error,
                ..
            } => {
                visitor.u64("attempts", u64::from(*attempts));
                visitor.u64("suppressed", *suppressed);
                visitor.string("error", error.clone());
            }
            Self::Recovered { attempts, .. } => {
                visitor.u64("attempts", u64::from(*attempts));
            }
            Self::IterationSummary {
                pg_ctl_lines_emitted,
                log_dir_files_tailed,
                log_dir_lines_emitted,
                dir_tailers,
                ..
            } => {
                visitor.u64("pg_ctl_lines_emitted", *pg_ctl_lines_emitted);
                visitor.u64("log_dir_files_tailed", *log_dir_files_tailed);
                visitor.u64("log_dir_lines_emitted", *log_dir_lines_emitted);
                visitor.usize("dir_tailers", *dir_tailers);
            }
        }
    }
}

impl SealedLogEvent for PostgresLineLogEvent {}

impl DomainLogEvent for PostgresLineLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::Json {
                source,
                severity,
                message,
                ..
            } => line_metadata(source, *severity, Cow::Owned(message.clone()), crate::logging::LogParser::PostgresJson),
            Self::Plain {
                source,
                severity,
                message,
                ..
            } => line_metadata(source, *severity, Cow::Owned(message.clone()), crate::logging::LogParser::PostgresPlain),
            Self::Unparsed {
                source,
                decoded_line,
            } => line_metadata(source, SeverityText::Info, Cow::Owned(decoded_line.clone()), crate::logging::LogParser::Raw),
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::Json { source, payload, .. } => {
                visitor.string("path", source.path.display().to_string());
                visitor.json("payload", payload.clone());
            }
            Self::Plain {
                source,
                level_raw,
                ..
            } => {
                visitor.string("path", source.path.display().to_string());
                visitor.string("level_raw", level_raw.clone());
            }
            Self::Unparsed {
                source,
                decoded_line,
            } => {
                visitor.string("path", source.path.display().to_string());
                visitor.bool("parse_failed", true);
                visitor.string("raw_line", decoded_line.clone());
            }
        }
    }
}

fn line_metadata(
    source: &PostgresLineSource,
    severity: SeverityText,
    message: Cow<'static, str>,
    parser: crate::logging::LogParser,
) -> LogEventMetadata {
    LogEventMetadata {
        severity,
        message,
        event_name: "postgres.line",
        event_domain: "postgres",
        event_result: LogEventResult::Ok,
        source: LogEventSource::new(
            source.producer,
            source.transport,
            parser,
            source.origin.clone(),
        ),
    }
}

const POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS: u64 = 30_000;
const POSTGRES_INGEST_MAX_BYTES_PER_FILE: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IngestErrorKey {
    stage: String,
    kind: String,
    path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RateLimitDecision {
    emit: bool,
    suppressed: u64,
}

#[derive(Clone, Debug)]
struct RateLimitState {
    last_emit_ms: u64,
    suppressed: u64,
}

#[derive(Clone, Debug)]
struct IngestErrorRateLimiter {
    window_ms: u64,
    by_key: BTreeMap<IngestErrorKey, RateLimitState>,
}

impl IngestErrorRateLimiter {
    fn new(window_ms: u64) -> Self {
        Self {
            window_ms,
            by_key: BTreeMap::new(),
        }
    }

    fn record(&mut self, key: IngestErrorKey, now_ms: u64) -> RateLimitDecision {
        match self.by_key.get_mut(&key) {
            None => {
                self.by_key.insert(
                    key,
                    RateLimitState {
                        last_emit_ms: now_ms,
                        suppressed: 0,
                    },
                );
                RateLimitDecision {
                    emit: true,
                    suppressed: 0,
                }
            }
            Some(entry) => {
                let elapsed_ms = now_ms.saturating_sub(entry.last_emit_ms);
                if elapsed_ms >= self.window_ms {
                    let suppressed = entry.suppressed;
                    entry.last_emit_ms = now_ms;
                    entry.suppressed = 0;
                    RateLimitDecision {
                        emit: true,
                        suppressed,
                    }
                } else {
                    entry.suppressed = entry.suppressed.saturating_add(1);
                    RateLimitDecision {
                        emit: false,
                        suppressed: 0,
                    }
                }
            }
        }
    }
}

pub(crate) async fn run(ctx: PostgresIngestWorkerCtx) -> Result<(), WorkerError> {
    let mut state = PostgresIngestWorkerState::new(&ctx.cfg);
    let mut limiter = IngestErrorRateLimiter::new(POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS);
    let mut consecutive_failures = 0u32;
    loop {
        if ctx.cfg.logging.postgres.enabled {
            match step_once(&ctx, &mut state).await {
                Ok(()) => {
                    if consecutive_failures > 0 {
                        ctx.log
                            .send(PostgresIngestLogEvent::Recovered {
                                origin: PostgresIngestOrigin::Run,
                                attempts: consecutive_failures,
                            })
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "postgres ingest recovered log send failed: {err}"
                                ))
                            })?;
                        consecutive_failures = 0;
                    }
                }
                Err(error) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    let now_ms = crate::logging::system_now_unix_millis();
                    let key = ingest_error_key_best_effort(&error);
                    let decision = limiter.record(key, now_ms);
                    if decision.emit {
                        ctx.log
                            .send(PostgresIngestLogEvent::StepOnceFailed {
                                origin: PostgresIngestOrigin::Run,
                                attempts: consecutive_failures,
                                suppressed: decision.suppressed,
                                error: error.to_string(),
                            })
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "postgres ingest error log send failed: {err}"
                                ))
                            })?;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(
            ctx.cfg.logging.postgres.poll_interval_ms,
        ))
        .await;
    }
}

fn ingest_error_key_best_effort(error: &WorkerError) -> IngestErrorKey {
    let msg = error.to_string();

    let mut stage = "unknown".to_string();
    let mut kind = "unknown".to_string();
    let mut path = "unknown".to_string();

    for token in msg.split_whitespace() {
        if stage == "unknown" {
            if let Some(value) = token.strip_prefix("stage=") {
                stage = value.to_string();
                continue;
            }
        }
        if kind == "unknown" {
            if let Some(value) = token.strip_prefix("kind=") {
                kind = value.to_string();
                continue;
            }
        }
        if path == "unknown" {
            if let Some(value) = token.strip_prefix("path=") {
                path = value.to_string();
                continue;
            }
        }
        if stage != "unknown" && kind != "unknown" && path != "unknown" {
            break;
        }
    }

    IngestErrorKey { stage, kind, path }
}

struct PostgresIngestWorkerState {
    pg_ctl_log: FileTailer,
    dir_tailers: DirTailers,
}

impl PostgresIngestWorkerState {
    fn new(cfg: &RuntimeConfig) -> Self {
        let pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.clone() {
            Some(path) => path,
            None => cfg.postgres_log_file(),
        };

        Self {
            pg_ctl_log: FileTailer::new(pg_ctl_log_file, StartPosition::Beginning),
            dir_tailers: DirTailers::default(),
        }
    }
}

async fn step_once(
    ctx: &PostgresIngestWorkerCtx,
    state: &mut PostgresIngestWorkerState,
) -> Result<(), WorkerError> {
    let max_bytes_per_file = POSTGRES_INGEST_MAX_BYTES_PER_FILE;
    let mut pg_ctl_lines_emitted: u64 = 0;
    let mut log_dir_lines_emitted: u64 = 0;
    let mut log_dir_files_tailed: u64 = 0;

    #[derive(Clone, Debug)]
    struct IterationIssue {
        stage: &'static str,
        kind: &'static str,
        path: String,
        error: String,
    }

    fn encode_path_token(path: &Path) -> String {
        path.display().to_string().replace(' ', "%20")
    }

    fn file_name_best_effort(path: &Path) -> String {
        match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => "log".to_string(),
        }
    }

    fn push_issue(
        issues: &mut Vec<IterationIssue>,
        stage: &'static str,
        kind: &'static str,
        path: &Path,
        error: WorkerError,
    ) {
        issues.push(IterationIssue {
            stage,
            kind,
            path: encode_path_token(path),
            error: error.to_string(),
        });
    }

    let mut issues: Vec<IterationIssue> = Vec::new();

    match state.pg_ctl_log.read_new_lines(max_bytes_per_file).await {
        Ok(pg_lines) => {
            for line in pg_lines {
                if let Err(err) = ctx.log.send(postgres_line_event(
                    LogProducer::Postgres,
                    LogTransport::FileTail,
                    "pg_ctl_log_file",
                    state.pg_ctl_log.path(),
                    line,
                )) {
                    push_issue(
                        &mut issues,
                        "pg_ctl_log_file.emit",
                        "log.emit_record",
                        state.pg_ctl_log.path(),
                        WorkerError::Message(err.to_string()),
                    );
                } else {
                    pg_ctl_lines_emitted = pg_ctl_lines_emitted.saturating_add(1);
                }
            }
        }
        Err(err) => {
            push_issue(
                &mut issues,
                "pg_ctl_log_file.read",
                "tailer.read_new_lines",
                state.pg_ctl_log.path(),
                err,
            );
        }
    }

    if let Some(dir) = ctx.cfg.logging.postgres.log_dir.as_ref() {
        if let Err(err) = discover_log_dir(&mut state.dir_tailers, dir).await {
            push_issue(&mut issues, "log_dir.discover", "read_dir", dir, err);
        }

        for (path, tailer) in state.dir_tailers.iter_mut() {
            log_dir_files_tailed = log_dir_files_tailed.saturating_add(1);
            let origin = format!("postgres_log_dir:{}", file_name_best_effort(path));
            match tailer.read_new_lines(max_bytes_per_file).await {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = ctx.log.send(postgres_line_event(
                            LogProducer::Postgres,
                            LogTransport::FileTail,
                            origin.as_str(),
                            tailer.path(),
                            line,
                        )) {
                            push_issue(
                                &mut issues,
                                "log_dir.emit",
                                "log.emit_record",
                                tailer.path(),
                                WorkerError::Message(err.to_string()),
                            );
                        } else {
                            log_dir_lines_emitted = log_dir_lines_emitted.saturating_add(1);
                        }
                    }
                }
                Err(err) => {
                    push_issue(
                        &mut issues,
                        "log_dir.read",
                        "tailer.read_new_lines",
                        tailer.path(),
                        err,
                    );
                }
            }
        }

        if ctx.cfg.logging.postgres.cleanup.enabled {
            let protected: Vec<&Path> = vec![state.pg_ctl_log.path()];

            match cleanup_log_dir(
                dir,
                &ctx.cfg.logging.postgres.cleanup,
                protected.as_slice(),
                SystemTime::now(),
            )
            .await
            {
                Ok(report) => {
                    if report.issue_count > 0 {
                        let stage = "log_dir.cleanup";
                        let kind = "cleanup.issues";
                        let error = WorkerError::Message(format!(
                            "cleanup had issues: issue_count={} first={}",
                            report.issue_count, report.first_issue
                        ));
                        push_issue(&mut issues, stage, kind, dir, error);
                    }
                }
                Err(err) => {
                    push_issue(&mut issues, "log_dir.cleanup", "cleanup.fatal", dir, err);
                }
            }
        }
    }

    if issues.is_empty() {
        ctx.log
            .send(PostgresIngestLogEvent::IterationSummary {
                origin: PostgresIngestOrigin::Run,
                pg_ctl_lines_emitted,
                log_dir_files_tailed,
                log_dir_lines_emitted,
                dir_tailers: state.dir_tailers.len(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("postgres ingest debug log send failed: {err}"))
            })?;
        return Ok(());
    }

    let first = match issues.first() {
        Some(first) => format!(
            "stage={} kind={} path={} error={}",
            first.stage, first.kind, first.path, first.error
        ),
        None => "stage=unknown kind=unknown path=unknown error=unknown".to_string(),
    };

    let mut extra = Vec::new();
    for issue in issues.iter().skip(1).take(2) {
        extra.push(format!(
            "stage={} kind={} path={} error={}",
            issue.stage, issue.kind, issue.path, issue.error
        ));
    }
    let extra_suffix = if extra.is_empty() {
        String::new()
    } else {
        format!(" extra=[{}]", extra.join(" | "))
    };

    Err(WorkerError::Message(format!(
        "postgres_ingest iteration_errors count={} {}{}",
        issues.len(),
        first,
        extra_suffix
    )))
}

async fn discover_log_dir(tailers: &mut DirTailers, dir: &Path) -> Result<(), WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("read_dir entry failed: {err}")))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "stage=log_dir.discover kind=file_type path={} error={err}",
                    path.display()
                )));
            }
        };
        if !is_file {
            continue;
        }

        let matches = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("log") | Some("json")
        );
        if !matches {
            continue;
        }

        let start = match path.file_name().and_then(|s| s.to_str()) {
            Some("postgres.stderr.log") | Some("postgres.stdout.log") => StartPosition::Beginning,
            _ => StartPosition::End,
        };
        tailers.ensure_file(path, start);
    }
    Ok(())
}

async fn cleanup_log_dir(
    dir: &Path,
    cleanup: &LogCleanupConfig,
    protected_paths: &[&Path],
    now: SystemTime,
) -> Result<CleanupReport, WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(CleanupReport::empty()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "cleanup read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    let protected_basenames: [&str; 3] = [
        "postgres.json",
        "postgres.stderr.log",
        "postgres.stdout.log",
    ];

    let mut issues: Vec<String> = Vec::new();
    let mut candidates = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("cleanup readdir entry failed: {err}")))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "stage=cleanup.file_type kind=file_type path={} error={err}",
                    path.display()
                )));
            }
        };
        if !is_file {
            continue;
        }

        let matches = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("log") | Some("json")
        );
        if !matches {
            continue;
        }

        let mut protected = false;
        for p in protected_paths {
            if path.as_path() == *p {
                protected = true;
                break;
            }
        }

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::new(),
        };
        if protected_basenames.contains(&file_name.as_str()) {
            protected = true;
        }

        let meta = match entry.metadata().await {
            Ok(meta) => meta,
            Err(err) => {
                protected = true;
                issues.push(format!(
                    "stage=cleanup.metadata kind=metadata path={} error={err}",
                    path.display()
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };
        let modified = match meta.modified() {
            Ok(modified) => Some(modified),
            Err(err) => {
                protected = true;
                issues.push(format!(
                    "stage=cleanup.modified kind=modified path={} error={err}",
                    path.display()
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };

        if !protected {
            let is_recent = match modified {
                Some(modified) => match now.duration_since(modified) {
                    Ok(age) => age.as_secs() <= cleanup.protect_recent_seconds,
                    Err(err) => {
                        issues.push(format!(
                            "stage=cleanup.age kind=duration_since path={} error={err}",
                            path.display()
                        ));
                        true
                    }
                },
                None => true,
            };
            if is_recent {
                protected = true;
            }
        }

        candidates.push((path, modified, protected));
    }

    let mut eligible = candidates
        .iter()
        .filter_map(|(path, modified, protected)| {
            if *protected {
                return None;
            }
            modified.map(|modified| (path.clone(), modified))
        })
        .collect::<Vec<_>>();

    eligible.sort_by(|a, b| {
        let by_time = a.1.cmp(&b.1);
        if by_time != std::cmp::Ordering::Equal {
            return by_time;
        }
        a.0.cmp(&b.0)
    });

    let mut to_remove: Vec<std::path::PathBuf> = Vec::new();

    if cleanup.max_files > 0 && (eligible.len() as u64) > cleanup.max_files {
        let remove_count = eligible.len().saturating_sub(cleanup.max_files as usize);
        for (path, _) in eligible.iter().take(remove_count) {
            to_remove.push(path.clone());
        }
    }

    if cleanup.max_age_seconds > 0 {
        for (path, modified) in eligible {
            match now.duration_since(modified) {
                Ok(age) => {
                    if age.as_secs() > cleanup.max_age_seconds {
                        to_remove.push(path);
                    }
                }
                Err(err) => {
                    issues.push(format!(
                        "stage=cleanup.age kind=duration_since path={} error={err}",
                        path.display()
                    ));
                }
            }
        }
    }

    for path in to_remove {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                issues.push(format!(
                    "stage=cleanup.remove_file kind=remove_file path={} error={err}",
                    path.display()
                ));
            }
        }
    }

    Ok(CleanupReport::from_issues(issues))
}

#[derive(Clone, Debug)]
struct CleanupReport {
    issue_count: usize,
    first_issue: String,
}

impl CleanupReport {
    fn empty() -> Self {
        Self {
            issue_count: 0,
            first_issue: "<none>".to_string(),
        }
    }

    fn from_issues(issues: Vec<String>) -> Self {
        let issue_count = issues.len();
        let first_issue = match issues.first() {
            Some(first) => first.to_string(),
            None => "<none>".to_string(),
        };
        Self {
            issue_count,
            first_issue,
        }
    }
}

fn postgres_line_event(
    producer: LogProducer,
    transport: LogTransport,
    origin: &str,
    path: &Path,
    line: Vec<u8>,
) -> PostgresLineLogEvent {
    let decoded = decode_line(&line);
    normalize_postgres_line(
        decoded.as_str(),
        PostgresLineSource {
            producer,
            transport,
            origin: format!("{origin}:{}", path.display()),
            path: path.to_path_buf(),
        },
    )
}

fn decode_line(line: &[u8]) -> String {
    match String::from_utf8(line.to_vec()) {
        Ok(s) => s,
        Err(err) => {
            let bytes = err.into_bytes();
            format!("non_utf8_bytes_hex={}", hex_encode(bytes.as_slice()))
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

fn normalize_postgres_line(line: &str, source: PostgresLineSource) -> PostgresLineLogEvent {
    if let Ok(value) = serde_json::from_str::<Value>(line) {
        if let Some(parsed) = normalize_postgres_json(value) {
            return PostgresLineLogEvent::Json {
                source,
                severity: parsed.severity,
                message: parsed.message,
                payload: parsed.payload,
            };
        }
    }

    if let Some(parsed) = normalize_postgres_plain(line) {
        return PostgresLineLogEvent::Plain {
            source,
            severity: parsed.severity,
            message: parsed.message,
            level_raw: parsed.level_raw,
        };
    }

    PostgresLineLogEvent::Unparsed {
        source,
        decoded_line: line.to_string(),
    }
}

struct ParsedLine {
    severity: SeverityText,
    message: String,
    payload: Value,
    level_raw: String,
}

fn normalize_postgres_json(value: Value) -> Option<ParsedLine> {
    let obj = value.as_object()?;
    let message = match obj.get("message").and_then(|v| v.as_str()) {
        Some(message) => message.to_string(),
        None => String::new(),
    };
    if message.trim().is_empty() {
        return None;
    }

    let severity_raw = obj
        .get("error_severity")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("severity").and_then(|v| v.as_str()));
    let severity_raw = severity_raw.map_or("INFO", |severity| severity);
    let severity = map_pg_severity(severity_raw);

    Some(ParsedLine {
        severity,
        message,
        payload: value,
        level_raw: String::new(),
    })
}

fn normalize_postgres_plain(line: &str) -> Option<ParsedLine> {
    // Example:
    // 2026-01-01 12:34:56.789 UTC [123] LOG:  message
    let bracket = line.find('[')?;
    let after_bracket = line[bracket..].find(']')?;
    let rest = line[bracket + after_bracket + 1..].trim_start();

    let (level, message) = rest.split_once(':')?;
    let level = level.trim();
    let message = message.trim_start().to_string();
    if level.is_empty() || message.is_empty() {
        return None;
    }
    let severity = map_pg_severity(level);

    Some(ParsedLine {
        severity,
        message,
        payload: Value::Null,
        level_raw: level.to_string(),
    })
}

fn map_pg_severity(raw: &str) -> SeverityText {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DEBUG" | "DEBUG1" | "DEBUG2" | "DEBUG3" | "DEBUG4" | "DEBUG5" => SeverityText::Debug,
        "INFO" | "NOTICE" | "LOG" => SeverityText::Info,
        "WARNING" => SeverityText::Warn,
        "ERROR" => SeverityText::Error,
        "FATAL" | "PANIC" => SeverityText::Fatal,
        _ => SeverityText::Info,
    }
}

pub(crate) fn build_ctx(cfg: RuntimeConfig, log: LogSender) -> PostgresIngestWorkerCtx {
    PostgresIngestWorkerCtx { cfg, log }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use serde_json::Value;
    use tokio::task::JoinHandle;

    use crate::config::{
        DebugConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig,
        PostgresLoggingConfig, RuntimeConfig,
    };
    use crate::logging::{LogParser, LogProducer, LogSender, LogTransport, SeverityText, TestSink};

    use crate::state::WorkerError;

    use super::{
        cleanup_log_dir, decode_line, ingest_error_key_best_effort, map_pg_severity,
        normalize_postgres_line, IngestErrorKey, IngestErrorRateLimiter, PostgresIngestLogEvent,
        PostgresIngestOrigin,
    };

    const REAL_INGEST_RETRY_SLEEP: Duration = Duration::from_millis(20);
    const REAL_PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(5);
    const REAL_PSQL_RETRY_SLEEP: Duration = Duration::from_millis(50);

    fn remove_dir_all_if_exists(path: &std::path::Path) -> Result<(), WorkerError> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(WorkerError::Message(err.to_string())),
        }
    }

    fn sample_runtime_config() -> RuntimeConfig {
        let baseline_logging = crate::dev_support::runtime_config::sample_postgres_logging_config();
        crate::dev_support::runtime_config::RuntimeConfigBuilder::new()
            .with_pg_hba(InlineOrPath::Inline {
                content: concat!("local all all trust\n", "host all all 127.0.0.1/32 trust\n",)
                    .to_string(),
            })
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..baseline_logging.cleanup
                    },
                    ..baseline_logging
                },
                ..crate::dev_support::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    struct RunningTestLog {
        sender: LogSender,
        sink: Arc<TestSink>,
        worker_task: JoinHandle<()>,
    }

    impl RunningTestLog {
        fn start() -> Self {
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
            let sink = Arc::new(TestSink::default());
            let sink_dyn: Arc<dyn super::super::LogSink> = sink.clone();
            let worker = super::super::LogWorker {
                receiver,
                backend: Arc::new(super::super::TracingBackend::new(sink_dyn)),
            };
            Self {
                sender: LogSender::new("host-a".to_string(), sender, SeverityText::Trace),
                sink,
                worker_task: tokio::spawn(worker.run()),
            }
        }

        fn sender(&self) -> LogSender {
            self.sender.clone()
        }

        async fn take(&self) -> Vec<crate::logging::LogRecord> {
            tokio::task::yield_now().await;
            self.sink.take()
        }
    }

    impl Drop for RunningTestLog {
        fn drop(&mut self) {
            self.worker_task.abort();
        }
    }

    fn materialize_record<E>(event: E) -> crate::logging::LogRecord
    where
        E: crate::logging::DomainLogEvent,
    {
        super::super::raw_record::QueuedRecord::from_event(1, "host-a".to_string(), event)
            .into_record()
    }

    fn sample_postgres_line_source() -> super::PostgresLineSource {
        super::PostgresLineSource {
            producer: LogProducer::Postgres,
            transport: LogTransport::FileTail,
            origin: "test".to_string(),
            path: PathBuf::from("/tmp/postgres.log"),
        }
    }

    fn normalized_postgres_record(raw: &str) -> crate::logging::LogRecord {
        materialize_record(normalize_postgres_line(raw, sample_postgres_line_source()))
    }

    fn start_test_log() -> RunningTestLog {
        RunningTestLog::start()
    }

    fn sample_postgres_ingest_failure_event(error: &WorkerError) -> PostgresIngestLogEvent {
        PostgresIngestLogEvent::StepOnceFailed {
            origin: PostgresIngestOrigin::Run,
            attempts: 2,
            suppressed: 7,
            error: error.to_string(),
        }
    }

    fn sample_non_utf8_postgres_line_event(path: &std::path::Path) -> super::PostgresLineLogEvent {
        super::postgres_line_event(
            LogProducer::Postgres,
            LogTransport::FileTail,
            "pg_ctl_log_file",
            path,
            vec![0xff_u8, 0x00, b'a', 0x80],
        )
    }

    #[test]
    fn ingest_error_rate_limiter_suppresses_and_reemits_with_count() {
        let mut limiter = IngestErrorRateLimiter::new(30_000);
        let key = IngestErrorKey {
            stage: "a".to_string(),
            kind: "b".to_string(),
            path: "c".to_string(),
        };

        let first = limiter.record(key.clone(), 1_000);
        assert_eq!(
            first,
            super::RateLimitDecision {
                emit: true,
                suppressed: 0
            }
        );

        let suppressed = limiter.record(key.clone(), 2_000);
        assert_eq!(
            suppressed,
            super::RateLimitDecision {
                emit: false,
                suppressed: 0
            }
        );

        let reemit = limiter.record(key, 31_000);
        assert_eq!(
            reemit,
            super::RateLimitDecision {
                emit: true,
                suppressed: 1
            }
        );
    }

    #[test]
    fn ingest_error_key_parsing_uses_first_stage_kind_path_tokens() {
        let err = WorkerError::Message(
            "postgres_ingest iteration_errors count=2 stage=first kind=k1 path=/a error=x extra=[stage=second kind=k2 path=/b error=y]"
                .to_string(),
        );
        let key = ingest_error_key_best_effort(&err);
        assert_eq!(key.stage, "first");
        assert_eq!(key.kind, "k1");
        assert_eq!(key.path, "/a");
    }

    #[test]
    fn step_failure_event_encodes_internal_error_record() {
        let err = WorkerError::Message("stage=x kind=y path=/z error=boom".to_string());
        let record = materialize_record(sample_postgres_ingest_failure_event(&err));

        assert_eq!(record.severity_text, SeverityText::Error);
        assert_eq!(record.source.origin, "postgres_ingest::run");
        assert_eq!(
            record.attributes.get("event.name"),
            Some(&Value::String("postgres_ingest.step_once_failed".to_string()))
        );
        assert_eq!(
            record.attributes.get("event.domain"),
            Some(&Value::String("postgres_ingest".to_string()))
        );
        assert_eq!(
            record.attributes.get("event.result"),
            Some(&Value::String("failed".to_string()))
        );
        assert_eq!(
            record.attributes.get("attempts"),
            Some(&Value::Number(serde_json::Number::from(2_u64)))
        );
        assert_eq!(
            record.attributes.get("suppressed"),
            Some(&Value::Number(serde_json::Number::from(7_u64)))
        );
    }

    #[test]
    fn map_pg_severity_maps_known_levels() {
        assert_eq!(map_pg_severity("ERROR"), SeverityText::Error);
        assert_eq!(map_pg_severity("warning"), SeverityText::Warn);
        assert_eq!(map_pg_severity("log"), SeverityText::Info);
    }

    #[test]
    fn normalize_postgres_line_parses_jsonlog() {
        let raw = r#"{"error_severity":"LOG","message":"hello from json"}"#;
        let record = normalized_postgres_record(raw);
        assert_eq!(record.source.parser, LogParser::PostgresJson);
        assert_eq!(record.message, "hello from json");
        assert_eq!(record.severity_text, SeverityText::Info);
        assert_eq!(record.severity_number, SeverityText::Info.number());
        assert_eq!(record.hostname, "host-a");
    }

    #[test]
    fn normalize_postgres_line_parses_plain() {
        let raw = "2026-03-04 01:02:03 UTC [123] ERROR:  something bad";
        let record = normalized_postgres_record(raw);
        assert_eq!(record.source.parser, LogParser::PostgresPlain);
        assert_eq!(record.severity_text, SeverityText::Error);
        assert_eq!(record.message, "something bad");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_failure() {
        let raw = "not a postgres log line";
        let record = normalized_postgres_record(raw);
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&serde_json::Value::String(raw.to_string()))
        );
    }

    #[test]
    fn decode_line_encodes_non_utf8_bytes_as_hex() {
        let bytes = [0xff_u8, 0x00, b'a', 0x80];
        assert_eq!(decode_line(bytes.as_slice()), "non_utf8_bytes_hex=ff006180");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_non_utf8_failure() {
        let bytes = [0xff_u8, 0x00, b'a', 0x80];
        let raw = decode_line(bytes.as_slice());
        let record = normalized_postgres_record(raw.as_str());
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
    }

    #[test]
    fn postgres_line_event_preserves_parse_failure_for_non_utf8() {
        let path = PathBuf::from("/tmp/pg.log");
        let record = materialize_record(sample_non_utf8_postgres_line_event(path.as_path()));
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
    }

    fn temp_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-logging-cleanup-{label}-{}-{unique}",
            std::process::id()
        ))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_enforces_max_files_and_protects_active_file() -> Result<(), WorkerError>
    {
        let dir = temp_dir("max-files");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let protected = dir.join("active.log");
        std::fs::write(&protected, b"active\n")
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..5 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        }

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 2,
                max_age_seconds: 365 * 24 * 60 * 60,
                protect_recent_seconds: 1,
            },
            &[protected.as_path()],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert_eq!(report.issue_count, 0);

        assert!(protected.exists());
        let mut remaining = 0usize;
        for entry in std::fs::read_dir(&dir).map_err(|err| WorkerError::Message(err.to_string()))? {
            let entry = entry.map_err(|err| WorkerError::Message(err.to_string()))?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                remaining = remaining.saturating_add(1);
            }
        }
        // protected + max_files
        assert!(remaining <= 3);

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_never_deletes_known_active_signals() -> Result<(), WorkerError> {
        let dir = temp_dir("protected-basenames");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let json = dir.join("postgres.json");
        let stderr = dir.join("postgres.stderr.log");
        let stdout = dir.join("postgres.stdout.log");
        std::fs::write(&json, b"{}\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        std::fs::write(&stderr, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        std::fs::write(&stdout, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..10 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        }

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 1,
                max_age_seconds: 365 * 24 * 60 * 60,
                protect_recent_seconds: 1,
            },
            &[],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert_eq!(report.issue_count, 0);

        assert!(json.exists());
        assert!(stderr.exists());
        assert!(stdout.exists());

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_surfaces_remove_failures() -> Result<(), WorkerError> {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_dir("remove-failure");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let old = dir.join("old.log");
        std::fs::write(&old, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        let mut perms = std::fs::metadata(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&dir, perms)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 1,
                max_age_seconds: 1,
                protect_recent_seconds: 1,
            },
            &[],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert!(report.issue_count > 0);
        assert!(old.exists());

        let mut perms = std::fs::metadata(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dir, perms)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    mod real_binary {
        use std::path::PathBuf;
        use std::time::Duration;

        use tokio::process::Command;
        use tokio::sync::mpsc;
        use tokio::time::Instant;

        use crate::dcs::{
            ClusterMemberView, ClusterView, DcsView, LeadershipObservation, MemberPostgresView,
            SwitchoverView,
        };
        use crate::dev_support::binaries::{
            require_pg16_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
        };
        use crate::dev_support::namespace::NamespaceGuard;
        use crate::dev_support::pg16::{
            prepare_pgdata_dir, spawn_pg16_for_vanilla_postgres, PgInstanceSpec,
        };
        use crate::dev_support::ports::allocate_ports;
        use crate::logging::LogRecord;
        use crate::process::jobs::{
            PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent, ShutdownMode,
        };
        use crate::process::state::{
            ProcessCadence, ProcessControlPlane, ProcessIntentRequest, ProcessNodeIdentity,
            ProcessObservedState, ProcessRuntime, ProcessRuntimePlan, ProcessState,
            ProcessStateChannel, ProcessWorkerBootstrap, ProcessWorkerCtx,
        };
        use crate::process::worker::{step_once as process_step_once, TokioCommandRunner};
        use crate::state::{
            new_state_channel, JobId, MemberId, TimelineId, WalLsn, WorkerError,
        };

        use super::super::{
            step_once as ingest_step_once, PostgresIngestWorkerCtx, PostgresIngestWorkerState,
        };
        use super::{
            sample_runtime_config, start_test_log, REAL_INGEST_RETRY_SLEEP,
            REAL_PROCESS_WORKER_POLL_INTERVAL, REAL_PSQL_RETRY_SLEEP,
        };

        async fn wait_for_process_idle_success(
            ctx: &mut ProcessWorkerCtx,
            job_id: &JobId,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            wait_for_process_idle_success_with_debug(ctx, job_id, timeout, None).await
        }

        async fn wait_for_process_idle_success_with_debug(
            ctx: &mut ProcessWorkerCtx,
            job_id: &JobId,
            timeout: Duration,
            debug_log_path: Option<&PathBuf>,
        ) -> Result<(), WorkerError> {
            let started = Instant::now();
            while started.elapsed() < timeout {
                process_step_once(ctx).await?;
                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &ctx.state_channel.current
                {
                    match outcome {
                        crate::process::state::JobOutcome::Success { id, .. } if *id == *job_id => {
                            return Ok(());
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if *id == *job_id =>
                        {
                            let debug_tail = match debug_log_path {
                                Some(path) => tail_file_best_effort(path, 60),
                                None => String::new(),
                            };
                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}{}",
                                job_id.0,
                                if debug_tail.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(
                                        "\n--- debug tail {} ---\n{debug_tail}",
                                        path_display(debug_log_path)
                                    )
                                }
                            )));
                        }
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(WorkerError::Message(format!(
                "timed out waiting for job {} success",
                job_id.0
            )))
        }

        fn path_display(path: Option<&PathBuf>) -> String {
            match path {
                Some(path) => path.display().to_string(),
                None => "<none>".to_string(),
            }
        }

        fn tail_file_best_effort(path: &PathBuf, max_lines: usize) -> String {
            let contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(err) => return format!("(failed to read {}: {err})", path.display()),
            };
            let mut lines = contents.lines().collect::<Vec<_>>();
            if lines.len() > max_lines {
                let start = lines.len().saturating_sub(max_lines);
                lines.drain(0..start);
            }
            lines.join("\n")
        }

        fn build_process_worker_ctx(
            cfg: &crate::config::RuntimeConfig,
            log: crate::logging::LogSender,
            dcs: DcsView,
            inbox: tokio::sync::mpsc::UnboundedReceiver<ProcessIntentRequest>,
        ) -> (
            ProcessWorkerCtx,
            crate::state::StateSubscriber<ProcessState>,
        ) {
            let initial = ProcessState::starting();
            let (publisher, subscriber) = new_state_channel(initial.clone());
            let (_cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
            let (_dcs_publisher, dcs_subscriber) = new_state_channel(dcs);
            (
                ProcessWorkerCtx::new(ProcessWorkerBootstrap {
                    cadence: ProcessCadence {
                        poll_interval: REAL_PROCESS_WORKER_POLL_INTERVAL,
                        now: Box::new(crate::process::worker::system_now_unix_millis),
                    },
                    config: cfg.process.clone(),
                    identity: ProcessNodeIdentity {
                        self_id: MemberId(cfg.cluster.member_id.clone()),
                    },
                    observed: ProcessObservedState {
                        runtime_config,
                        dcs: dcs_subscriber,
                    },
                    plan: ProcessRuntimePlan::from_config(cfg),
                    state_channel: ProcessStateChannel {
                        current: initial,
                        publisher,
                        last_rejection: None,
                    },
                    control: ProcessControlPlane {
                        inbox,
                        inbox_disconnected_logged: false,
                        active_runtime: None,
                    },
                    runtime: ProcessRuntime {
                        log,
                        capture_subprocess_output: true,
                        command_runner: Box::new(TokioCommandRunner),
                    },
                }),
                subscriber,
            )
        }

        fn is_transient_psql_failure(stderr: &str) -> bool {
            let normalized = stderr.to_ascii_lowercase();
            normalized.contains("the database system is starting up")
                || normalized.contains("the database system is shutting down")
                || normalized.contains("not yet accepting connections")
                || normalized.contains("could not connect to server")
                || normalized.contains("connection refused")
        }

        async fn run_psql_query_with_retry(
            psql_bin: &PathBuf,
            port: u16,
            query: &str,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            let deadline = Instant::now() + timeout;
            let mut last_stderr = String::new();
            let mut last_stdout = String::new();

            while Instant::now() < deadline {
                let mut cmd = Command::new(psql_bin);
                cmd.arg("-h")
                    .arg("127.0.0.1")
                    .arg("-p")
                    .arg(port.to_string())
                    .arg("-U")
                    .arg("postgres")
                    .arg("-d")
                    .arg("postgres")
                    .arg("-c")
                    .arg(query);

                let output = cmd
                    .output()
                    .await
                    .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;

                if output.status.success() {
                    return Ok(());
                }

                last_stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                last_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                if !is_transient_psql_failure(&last_stderr) {
                    return Err(WorkerError::Message(format!(
                        "psql exited unsuccessfully: {} (non-transient)\n--- stdout ---\n{}\n--- stderr ---\n{}",
                        output.status,
                        last_stdout,
                        last_stderr
                    )));
                }

                tokio::time::sleep(REAL_PSQL_RETRY_SLEEP).await;
            }

            Err(WorkerError::Message(format!(
                "timed out waiting for psql readiness after {:?}\n--- last stdout ---\n{}\n--- last stderr ---\n{}",
                timeout, last_stdout, last_stderr
            )))
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ingests_jsonlog_and_stderr_files_from_real_postgres() -> Result<(), WorkerError> {
            let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
            let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
            let psql_bin = require_pg16_bin_for_real_tests("psql")?;

            let guard = NamespaceGuard::new("log-jsonlog-stderr")?;
            let ns = guard.namespace()?;

            let data_dir = prepare_pgdata_dir(ns, "node-a")?;
            let mut reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];
            let socket_dir = ns.child_dir("pg16/node-a/socket");
            let log_dir = ns.child_dir("logs/pg16-node-a");

            let jsonlog_path = log_dir.join("postgres.json");
            std::fs::create_dir_all(&log_dir).map_err(|err| {
                WorkerError::Message(format!(
                    "create postgres ingest log dir {} failed: {err}",
                    log_dir.display()
                ))
            })?;
            std::fs::write(&jsonlog_path, b"").map_err(|err| {
                WorkerError::Message(format!(
                    "seed postgres ingest jsonlog file {} failed: {err}",
                    jsonlog_path.display()
                ))
            })?;

            let conf_lines = vec![
                "logging_collector = on".to_string(),
                "log_destination = 'jsonlog,stderr'".to_string(),
                format!("log_directory = '{}'", log_dir.display()),
                "log_filename = 'postgres.json'".to_string(),
                "log_statement = 'all'".to_string(),
            ];

            let spec = PgInstanceSpec {
                postgres_bin,
                initdb_bin,
                data_dir,
                socket_dir,
                log_dir: log_dir.clone(),
                port,
                startup_timeout: Duration::from_secs(10),
            };
            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            // This test validates raw PostgreSQL log emission and ingest parsing, not
            // pgtuskmaster-managed startup ownership, so it uses the explicit
            // vanilla-Postgres config exception path.
            let mut pg = spawn_pg16_for_vanilla_postgres(spec, &conf_lines).await?;

            let mut cfg = sample_runtime_config();
            cfg.logging.postgres.log_dir = Some(log_dir);
            cfg.logging.postgres.cleanup.enabled = false;
            cfg.postgres.paths.log_file = Some(ns.child_dir("runtime/pg_ctl.log"));

            let test_log = start_test_log();
            let ctx = PostgresIngestWorkerCtx {
                cfg,
                log: test_log.sender(),
            };
            let mut state = PostgresIngestWorkerState::new(&ctx.cfg);

            // Prime ingestion offsets and then generate logs.
            ingest_step_once(&ctx, &mut state).await?;

            run_psql_query_with_retry(&psql_bin, port, "SELECT 1;", Duration::from_secs(10))
                .await?;

            let deadline = Instant::now() + Duration::from_secs(3);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ctx, &mut state).await?;
                collected.extend(test_log.take().await);
                let saw_json = collected
                    .iter()
                    .any(|r| r.source.parser == crate::logging::LogParser::PostgresJson);
                let saw_stderr = collected
                    .iter()
                    .any(|r| r.source.origin.contains("postgres.stderr.log"));
                if saw_json && saw_stderr {
                    pg.shutdown().await?;
                    return Ok(());
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            pg.shutdown().await?;
            drop(reservation);
            Err(WorkerError::Message(
                "timed out waiting for jsonlog+stderr ingestion".to_string(),
            ))
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ingests_pg_ctl_log_file_and_captures_pg_tool_output() -> Result<(), WorkerError> {
            let binaries = require_pg16_process_binaries_for_real_tests()?;

            let guard = NamespaceGuard::new("log-pgctl")?;
            let ns = guard.namespace()?;

            let mut reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];

            let data_dir = prepare_pgdata_dir(ns, "node-a")?;
            let socket_dir = ns.child_dir("sock");
            let log_file = ns.child_dir("runtime/pg_ctl.log");
            let log_dir = ns.child_dir("logs/pg16-node-a");
            std::fs::create_dir_all(&socket_dir)
                .map_err(|err| WorkerError::Message(format!("create socket_dir failed: {err}")))?;
            if let Some(parent) = log_file.parent() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    WorkerError::Message(format!("create log file parent failed: {err}"))
                })?;
            }
            std::fs::create_dir_all(&log_dir)
                .map_err(|err| WorkerError::Message(format!("create log_dir failed: {err}")))?;
            let jsonlog_path = log_dir.join("postgres.json");
            std::fs::write(&jsonlog_path, b"")
                .map_err(|err| WorkerError::Message(format!("seed jsonlog failed: {err}")))?;

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries.clone();
            cfg.postgres.paths.data_dir = data_dir.clone();
            cfg.postgres.paths.socket_dir = Some(socket_dir.clone());
            cfg.postgres.network.listen_port = port;
            cfg.postgres.paths.log_file = Some(log_file.clone());
            cfg.postgres
                .extra_gucs
                .insert("log_filename".to_string(), "postgres.json".to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_directory".to_string(), log_dir.display().to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_statement".to_string(), "all".to_string());
            cfg.logging.postgres.log_dir = Some(log_dir.clone());
            cfg.logging.postgres.cleanup.enabled = false;

            let test_log = start_test_log();

            let (tx, rx) = mpsc::unbounded_channel();
            let (mut process_ctx, _process_state_subscriber) = build_process_worker_ctx(
                &cfg,
                test_log.sender(),
                DcsView::starting(),
                rx,
            );

            let ingest_ctx = PostgresIngestWorkerCtx {
                cfg,
                log: test_log.sender(),
            };
            let mut ingest_state = PostgresIngestWorkerState::new(&ingest_ctx.cfg);

            let bootstrap_id = JobId("bootstrap".to_string());
            tx.send(ProcessIntentRequest {
                id: bootstrap_id.clone(),
                intent: ProcessIntent::Bootstrap,
            })
            .map_err(|_| WorkerError::Message("send bootstrap job failed".to_string()))?;

            wait_for_process_idle_success(&mut process_ctx, &bootstrap_id, Duration::from_secs(30))
                .await?;

            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            let start_id = JobId("start".to_string());
            tx.send(ProcessIntentRequest {
                id: start_id.clone(),
                intent: ProcessIntent::Start(PostgresStartIntent::Primary),
            })
            .map_err(|_| WorkerError::Message("send start job failed".to_string()))?;

            let started = Instant::now();
            let mut collected_for_debug: Vec<LogRecord> = Vec::new();
            while started.elapsed() < Duration::from_secs(60) {
                process_step_once(&mut process_ctx).await?;
                collected_for_debug.extend(test_log.take().await);

                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &process_ctx.state_channel.current
                {
                    match outcome {
                        crate::process::state::JobOutcome::Success { id, .. }
                            if *id == start_id =>
                        {
                            break;
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if *id == start_id =>
                        {
                            let pg_ctl_tail = tail_file_best_effort(&log_file, 120);
                            let postgres_json_tail = tail_file_best_effort(&jsonlog_path, 120);
                            let postmaster_pid =
                                tail_file_best_effort(&data_dir.join("postmaster.pid"), 60);

                            let mut pg_tool_lines = Vec::new();
                            for record in &collected_for_debug {
                                if record.source.producer != crate::logging::LogProducer::PgTool {
                                    continue;
                                }
                                let job_kind = record
                                    .attributes
                                    .get("job.kind")
                                    .and_then(|v| v.as_str())
                                    .map_or("<none>", |value| value);
                                let job_id_attr = record
                                    .attributes
                                    .get("job.id")
                                    .and_then(|v| v.as_str())
                                    .map_or("<none>", |value| value);
                                if job_kind != "start_postgres"
                                    && job_id_attr != start_id.0.as_str()
                                {
                                    continue;
                                }
                                pg_tool_lines.push(format!(
                                    "{:?} {}: {}",
                                    record.source.transport, record.source.origin, record.message
                                ));
                            }
                            if pg_tool_lines.len() > 60 {
                                let start = pg_tool_lines.len().saturating_sub(60);
                                pg_tool_lines.drain(0..start);
                            }
                            let pg_tool_debug = if pg_tool_lines.is_empty() {
                                "(no captured pg_tool stdout/stderr lines for start_postgres)"
                                    .to_string()
                            } else {
                                pg_tool_lines.join("\n")
                            };

                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}\n--- pg_ctl log tail {} ---\n{}\n--- postgres jsonlog tail {} ---\n{}\n--- postmaster.pid tail {} ---\n{}\n--- captured pg_tool output (start_postgres) ---\n{}",
                                start_id.0,
                                log_file.display(),
                                pg_ctl_tail,
                                jsonlog_path.display(),
                                postgres_json_tail,
                                data_dir.join("postmaster.pid").display(),
                                postmaster_pid,
                                pg_tool_debug
                            )));
                        }
                        _ => {}
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            if started.elapsed() >= Duration::from_secs(60) {
                return Err(WorkerError::Message(
                    "timed out waiting for start_postgres job success".to_string(),
                ));
            }

            // Pump ingestion a bit to collect pg_ctl log lines.
            let psql_bin = binaries.overrides.psql.clone().ok_or_else(|| {
                WorkerError::Message("test process binaries missing psql override".to_string())
            })?;
            let mut cmd = Command::new(psql_bin);
            cmd.arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(port.to_string())
                .arg("-U")
                .arg("postgres")
                .arg("-d")
                .arg("postgres")
                .arg("-c")
                .arg("SELECT 1;");
            let status = cmd
                .status()
                .await
                .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;
            if !status.success() {
                return Err(WorkerError::Message(format!(
                    "psql pg_switch_wal exited unsuccessfully: {status}"
                )));
            }

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ingest_ctx, &mut ingest_state).await?;
                process_step_once(&mut process_ctx).await?;
                collected.extend(test_log.take().await);
                let saw_pg_ctl_log = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::Postgres
                        && r.source.origin.contains("pg_ctl_log_file")
                });
                let saw_pg_tool = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PgTool
                        && (r.source.transport == crate::logging::LogTransport::ChildStdout
                            || r.source.transport == crate::logging::LogTransport::ChildStderr)
                });
                let saw_jsonlog = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::Postgres
                        && r.source.parser == crate::logging::LogParser::PostgresJson
                });
                if saw_pg_ctl_log && saw_pg_tool && saw_jsonlog {
                    break;
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            let stop_id = JobId("stop".to_string());
            tx.send(ProcessIntentRequest {
                id: stop_id.clone(),
                intent: ProcessIntent::Demote(ShutdownMode::Fast),
            })
            .map_err(|_| WorkerError::Message("send stop job failed".to_string()))?;
            wait_for_process_idle_success(&mut process_ctx, &stop_id, Duration::from_secs(30))
                .await?;

            // One more ingestion pass after shutdown to catch any final flushes.
            ingest_step_once(&ingest_ctx, &mut ingest_state).await?;

            let mut all_records = collected;
            all_records.extend(test_log.take().await);

            let saw_pg_ctl_log = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::Postgres
                    && r.source.origin.contains("pg_ctl_log_file")
            });
            let saw_pg_tool = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::PgTool
                    && r.attributes
                        .get("job.kind")
                        .and_then(|v| v.as_str())
                        .is_some()
            });
            let saw_jsonlog = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::Postgres
                    && r.source.parser == crate::logging::LogParser::PostgresJson
            });
            if !saw_pg_ctl_log {
                return Err(WorkerError::Message(
                    "missing ingested pg_ctl log file records".to_string(),
                ));
            }
            if !saw_pg_tool {
                return Err(WorkerError::Message(
                    "missing captured pg tool stdout/stderr records".to_string(),
                ));
            }
            if !saw_jsonlog {
                return Err(WorkerError::Message(
                    "missing ingested postgres jsonlog records".to_string(),
                ));
            }

            drop(reservation);
            Ok(())
        }

        #[tokio::test(flavor = "current_thread")]
        async fn captures_helper_binary_stdout_stderr_on_failure() -> Result<(), WorkerError> {
            let binaries = require_pg16_process_binaries_for_real_tests()?;

            let guard = NamespaceGuard::new("log-pgtool")?;
            let ns = guard.namespace()?;

            let data_dir = ns.child_dir("pg_basebackup/out");
            std::fs::create_dir_all(&data_dir)
                .map_err(|err| WorkerError::Message(format!("create data_dir failed: {err}")))?;

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries;

            let test_log = start_test_log();

            let (tx, rx) = mpsc::unbounded_channel();
            let dcs = DcsView::Coordinated(ClusterView::new(
                std::collections::BTreeMap::from([(
                    MemberId("node-b".to_string()),
                    ClusterMemberView::new(
                        MemberPostgresView::Primary {
                            readiness: crate::pginfo::state::Readiness::Ready,
                            system_identifier: None,
                            committed_wal: crate::state::ObservedWalPosition {
                                timeline: Some(TimelineId(1)),
                                lsn: WalLsn(0),
                            },
                        },
                        crate::state::PgTcpTarget::new("127.0.0.1".to_string(), 9)
                            .map_err(|err| WorkerError::Message(format!("test dcs target failed: {err}")))?,
                    ),
                )]),
                LeadershipObservation::Open,
                SwitchoverView::None,
            ));
            let (mut ctx, _process_state_subscriber) =
                build_process_worker_ctx(&cfg, test_log.sender(), dcs, rx);

            let job_id = JobId("basebackup-fail".to_string());
            tx.send(ProcessIntentRequest {
                id: job_id.clone(),
                intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: MemberId("node-b".to_string()),
                }),
            })
            .map_err(|_| WorkerError::Message("send basebackup job failed".to_string()))?;

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                process_step_once(&mut ctx).await?;
                collected.extend(test_log.take().await);
                let saw_stderr = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PgTool
                        && r.source.transport == crate::logging::LogTransport::ChildStderr
                        && r.attributes.get("job.kind").and_then(|v| v.as_str())
                            == Some("basebackup")
                });
                if saw_stderr {
                    return Ok(());
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            Err(WorkerError::Message(
                "timed out waiting for captured pg_basebackup stderr".to_string(),
            ))
        }
    }
}


===== src/runtime/node.rs =====
use std::{path::Path, time::Duration};

use thiserror::Error;

use crate::{
    config::{load_runtime_config, validate_runtime_config, ConfigError, RuntimeConfig},
    process::state::ProcessRuntimePlan,
    state::{new_state_channel, ClusterName, MemberId, NodeIdentity, ScopeName},
};

use super::log_event::{RuntimeLogEvent, RuntimeLogOrigin, RuntimeNodeIdentity};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("startup planning failed: {0}")]
    StartupPlanning(String),
    #[error("startup execution failed: {0}")]
    StartupExecution(String),
    #[error("api bind failed at `{listen_addr}`: {message}")]
    ApiBind {
        listen_addr: std::net::SocketAddr,
        message: String,
    },
    #[error("worker failed: {0}")]
    Worker(String),
    #[error("time error: {0}")]
    Time(String),
}

fn runtime_startup_event(cfg: &RuntimeConfig, startup_run_id: &str) -> RuntimeLogEvent {
    RuntimeLogEvent::StartupEntered {
        origin: RuntimeLogOrigin::RunNodeFromConfig,
        identity: RuntimeNodeIdentity {
            scope: cfg.cluster.scope.clone(),
            member_id: cfg.cluster.member_id.clone(),
        },
        startup_run_id: startup_run_id.to_string(),
        logging_level: cfg.logging.level,
    }
}

pub async fn run_node_from_config_path(path: &Path) -> Result<(), RuntimeError> {
    let cfg = load_runtime_config(path)?;
    run_node_from_config(cfg).await
}

pub async fn run_node_from_config(cfg: RuntimeConfig) -> Result<(), RuntimeError> {
    validate_runtime_config(&cfg)?;

    let logging = crate::logging::bootstrap(&cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("logging bootstrap failed: {err}"))
    })?;
    let log = logging.sender.clone();
    let worker = logging.worker;
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    log.send(runtime_startup_event(&cfg, startup_run_id.as_str()))
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
        })?;

    let process_plan = ProcessRuntimePlan::from_config(&cfg);
    process_plan.ensure_start_paths().map_err(|err| {
        RuntimeError::StartupExecution(format!("process start path preparation failed: {err}"))
    })?;

    run_workers(cfg, process_plan, log, worker).await
}

async fn run_workers(
    cfg: RuntimeConfig,
    process_plan: ProcessRuntimePlan,
    log: crate::logging::LogSender,
    log_worker: crate::logging::LogWorker,
) -> Result<(), RuntimeError> {
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone());
    let identity = NodeIdentity {
        cluster_name: ClusterName(cfg.cluster.name.clone()),
        scope: ScopeName(cfg.cluster.scope.clone()),
        member_id: MemberId(cfg.cluster.member_id.clone()),
    };
    let worker_poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);

    let pginfo = crate::pginfo::startup::bootstrap(crate::pginfo::startup::PgInfoRuntimeRequest {
        self_id: identity.member_id.clone(),
        probe: crate::pginfo::state::PgProbeTarget::local_from_config(&cfg, &process_plan),
        poll_interval: worker_poll_interval,
        log: log.clone(),
    });

    let dcs = crate::dcs::startup::bootstrap(crate::dcs::startup::DcsRuntimeRequest {
        identity: identity.clone(),
        endpoints: cfg.dcs.endpoints.clone(),
        client: cfg.dcs.client.clone(),
        poll_interval: worker_poll_interval,
        member_ttl_ms: cfg.ha.lease_ttl_ms,
        advertised: crate::dcs::startup::DcsAdvertisedEndpoints::from_config(&cfg)
            .map_err(|err| RuntimeError::Worker(format!("dcs advertisement build failed: {err}")))?,
        pg_subscriber: pginfo.state.clone(),
        log: log.clone(),
    })
    .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;

    let process =
        crate::process::startup::bootstrap(crate::process::startup::ProcessRuntimeRequest {
            identity: identity.clone(),
            runtime_config: cfg_subscriber.clone(),
            dcs_subscriber: dcs.state.clone(),
            plan: process_plan,
            config: cfg.process.clone(),
            capture_subprocess_output: cfg.logging.capture_subprocess_output,
            log: log.clone(),
        });

    let ha = crate::ha::startup::bootstrap(crate::ha::startup::HaRuntimeRequest {
        identity: identity.clone(),
        poll_interval: worker_poll_interval,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pginfo.state.clone(),
        dcs_subscriber: dcs.state.clone(),
        process_subscriber: process.state.clone(),
        process_control: process.control.clone(),
        dcs_handle: dcs.handle.clone(),
    });

    let api = crate::api::startup::bootstrap(crate::api::startup::ApiRuntimeRequest {
        identity,
        runtime_config: cfg_subscriber,
        dcs_handle: dcs.handle.clone(),
        observed_state: crate::api::worker::ApiObservedState::Live {
            pg: pginfo.state.clone(),
            process: process.state.clone(),
            dcs: dcs.state.clone(),
            ha: ha.state.clone(),
        },
        log: log.clone(),
    })
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    let (
        (),
        pginfo_result,
        dcs_result,
        process_result,
        ingest_result,
        ha_result,
        api_result,
    ) = tokio::join!(
        log_worker.run(),
        pginfo.worker.run(),
        dcs.worker.run(),
        process.worker.run(),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        ha.worker.run(),
        api.worker.run(),
    );

    pginfo_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    dcs_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    process_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    ingest_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    ha_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;
    api_result.map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}


===== src/runtime/log_event.rs =====
use std::borrow::Cow;

use crate::config::LogLevel;
use crate::logging::{
    DomainLogEvent, LogEventMetadata, LogEventResult, LogEventSource, LogFieldVisitor,
    SealedLogEvent, SeverityText,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeLogOrigin {
    RunNodeFromConfig,
}

impl RuntimeLogOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::RunNodeFromConfig => "runtime::run_node_from_config",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeNodeIdentity {
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeLogEvent {
    StartupEntered {
        origin: RuntimeLogOrigin,
        identity: RuntimeNodeIdentity,
        startup_run_id: String,
        logging_level: LogLevel,
    },
}

impl SealedLogEvent for RuntimeLogEvent {}

impl DomainLogEvent for RuntimeLogEvent {
    fn metadata(&self) -> LogEventMetadata {
        match self {
            Self::StartupEntered { origin, .. } => LogEventMetadata {
                severity: SeverityText::Info,
                message: Cow::Borrowed("runtime starting"),
                event_name: "runtime.startup.entered",
                event_domain: "runtime",
                event_result: LogEventResult::Ok,
                source: LogEventSource::app(origin.label()),
            },
        }
    }

    fn write_fields(&self, visitor: &mut dyn LogFieldVisitor) {
        match self {
            Self::StartupEntered {
                identity,
                startup_run_id,
                logging_level,
                ..
            } => {
                visitor.string("scope", identity.scope.clone());
                visitor.string("member_id", identity.member_id.clone());
                visitor.string("startup_run_id", startup_run_id.clone());
                visitor.str("logging.level", log_level_label(*logging_level));
            }
        }
    }
}

fn log_level_label(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
        LogLevel::Fatal => "fatal",
    }
}


===== src/process/state.rs =====
use std::{fs, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    config::{PostgresRoleName, ProcessConfig, RoleAuthConfig, RuntimeConfig},
    dcs::DcsView,
    logging::LogSender,
    pginfo::state::PgSslMode,
    state::{
        JobId, MemberId, StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus,
    },
};

use super::jobs::{
    ActiveJob, ActiveJobKind, BaseBackupSpec, BootstrapSpec, DemoteSpec, PgRewindSpec,
    ProcessCommandRunner, ProcessError, ProcessHandle, ProcessIntent, ProcessLogIdentity,
    PromoteSpec, StartPostgresSpec,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessState {
    Idle {
        worker: WorkerStatus,
        last_outcome: Option<JobOutcome>,
    },
    Running {
        worker: WorkerStatus,
        active: ActiveJob,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessExecutionKind {
    Bootstrap(BootstrapSpec),
    BaseBackup(BaseBackupSpec),
    PgRewind(PgRewindSpec),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
    StartPostgres(StartPostgresSpec),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessIntentRequest {
    pub(crate) id: JobId,
    pub(crate) intent: ProcessIntent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessExecutionRequest {
    pub(crate) id: JobId,
    pub(crate) kind: ProcessExecutionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobRejection {
    pub(crate) id: JobId,
    pub(crate) error: ProcessError,
    pub(crate) rejected_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
    Success {
        id: JobId,
        job_kind: ActiveJobKind,
        finished_at: UnixMillis,
    },
    Failure {
        id: JobId,
        job_kind: ActiveJobKind,
        error: ProcessError,
        finished_at: UnixMillis,
    },
    Timeout {
        id: JobId,
        job_kind: ActiveJobKind,
        finished_at: UnixMillis,
    },
}

pub(crate) struct ActiveRuntime {
    pub(crate) request: ProcessExecutionRequest,
    pub(crate) deadline_at: UnixMillis,
    pub(crate) handle: Box<dyn ProcessHandle>,
    pub(crate) log_identity: ProcessLogIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresPaths {
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresRuntime {
    pub(crate) paths: ManagedPostgresPaths,
    pub(crate) port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MandatoryPostgresRoleCredential {
    pub(crate) username: PostgresRoleName,
    pub(crate) auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MandatoryPostgresRuntimeRoles {
    pub(crate) superuser: MandatoryPostgresRoleCredential,
    pub(crate) replicator: MandatoryPostgresRoleCredential,
    pub(crate) rewinder: MandatoryPostgresRoleCredential,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicaAccessRuntime {
    pub(crate) roles: MandatoryPostgresRuntimeRoles,
    pub(crate) dbname: String,
    pub(crate) ssl_mode: PgSslMode,
    pub(crate) ssl_root_cert: Option<PathBuf>,
    pub(crate) connect_timeout_s: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessRuntimePlan {
    pub(crate) postgres: ManagedPostgresRuntime,
    pub(crate) replica_access: ReplicaAccessRuntime,
}

pub(crate) struct ProcessWorkerBootstrap {
    pub(crate) cadence: ProcessCadence,
    pub(crate) config: ProcessConfig,
    pub(crate) identity: ProcessNodeIdentity,
    pub(crate) observed: ProcessObservedState,
    pub(crate) plan: ProcessRuntimePlan,
    pub(crate) state_channel: ProcessStateChannel,
    pub(crate) control: ProcessControlPlane,
    pub(crate) runtime: ProcessRuntime,
}

pub(crate) struct ProcessWorkerCtx {
    pub(crate) cadence: ProcessCadence,
    pub(crate) config: ProcessConfig,
    pub(crate) identity: ProcessNodeIdentity,
    pub(crate) observed: ProcessObservedState,
    pub(crate) plan: ProcessRuntimePlan,
    pub(crate) state_channel: ProcessStateChannel,
    pub(crate) control: ProcessControlPlane,
    pub(crate) runtime: ProcessRuntime,
}

pub(crate) struct ProcessCadence {
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessNodeIdentity {
    pub(crate) self_id: MemberId,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessObservedState {
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs: StateSubscriber<DcsView>,
}

pub(crate) struct ProcessStateChannel {
    pub(crate) current: ProcessState,
    pub(crate) publisher: StatePublisher<ProcessState>,
    pub(crate) last_rejection: Option<ProcessJobRejection>,
}

pub(crate) struct ProcessControlPlane {
    pub(crate) inbox: UnboundedReceiver<ProcessIntentRequest>,
    pub(crate) inbox_disconnected_logged: bool,
    pub(crate) active_runtime: Option<ActiveRuntime>,
}

pub(crate) struct ProcessRuntime {
    pub(crate) log: LogSender,
    pub(crate) capture_subprocess_output: bool,
    pub(crate) command_runner: Box<dyn ProcessCommandRunner>,
}

impl ProcessWorkerCtx {
    pub(crate) fn new(bootstrap: ProcessWorkerBootstrap) -> Self {
        let ProcessWorkerBootstrap {
            cadence,
            config,
            identity,
            observed,
            plan,
            state_channel,
            control,
            runtime,
        } = bootstrap;
        Self {
            cadence,
            config,
            identity,
            observed,
            plan,
            state_channel,
            control,
            runtime,
        }
    }
}

impl ProcessRuntimePlan {
    pub(crate) fn from_config(cfg: &RuntimeConfig) -> Self {
        Self {
            postgres: ManagedPostgresRuntime {
                paths: ManagedPostgresPaths {
                    data_dir: cfg.postgres.paths.data_dir.clone(),
                    socket_dir: cfg.postgres_socket_dir(),
                    log_file: cfg.postgres_log_file(),
                },
                port: cfg.postgres.network.listen_port,
            },
            replica_access: ReplicaAccessRuntime {
                roles: MandatoryPostgresRuntimeRoles {
                    superuser: MandatoryPostgresRoleCredential {
                        username: cfg.postgres.roles.mandatory.superuser.username.clone(),
                        auth: cfg.postgres.roles.mandatory.superuser.auth.clone(),
                    },
                    replicator: MandatoryPostgresRoleCredential {
                        username: cfg.postgres.roles.mandatory.replicator.username.clone(),
                        auth: cfg.postgres.roles.mandatory.replicator.auth.clone(),
                    },
                    rewinder: MandatoryPostgresRoleCredential {
                        username: cfg.postgres.roles.mandatory.rewinder.username.clone(),
                        auth: cfg.postgres.roles.mandatory.rewinder.auth.clone(),
                    },
                },
                dbname: cfg.postgres.rewind.database.clone(),
                ssl_mode: cfg.postgres.rewind.transport.ssl_mode,
                ssl_root_cert: cfg
                    .postgres
                    .rewind
                    .transport
                    .ca_cert
                    .as_ref()
                    .and_then(|source| match source {
                        crate::config::InlineOrPath::Path(path)
                        | crate::config::InlineOrPath::PathConfig { path } => Some(path.clone()),
                        crate::config::InlineOrPath::Inline { .. } => None,
                    }),
                connect_timeout_s: cfg.postgres.connect_timeout_s,
            },
        }
    }

}

impl ProcessRuntimePlan {
    pub(crate) fn ensure_start_paths(&self) -> Result<(), ProcessError> {
        let data_dir = &self.postgres.paths.data_dir;
        if let Some(parent) = data_dir.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "failed to create postgres data dir parent `{}`: {err}",
                    parent.display()
                ))
            })?;
        }

        fs::create_dir_all(data_dir).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres data dir `{}`: {err}",
                data_dir.display()
            ))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "failed to set postgres data dir permissions on `{}`: {err}",
                    data_dir.display()
                ))
            })?;
        }

        fs::create_dir_all(&self.postgres.paths.socket_dir).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres socket dir `{}`: {err}",
                self.postgres.paths.socket_dir.display()
            ))
        })?;

        if let Some(log_parent) = self.postgres.paths.log_file.parent() {
            fs::create_dir_all(log_parent).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "failed to create postgres log dir `{}`: {err}",
                    log_parent.display()
                ))
            })?;
        }

        Ok(())
    }
}

impl ProcessState {
    pub(crate) fn starting() -> Self {
        Self::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        }
    }
}


===== docs/tmp/verbose_extra_context/process-logging-boundary.md =====
# Verbose Context: Process Logging Boundary

This file is raw factual context for documentation drafting.

The logging subsystem is now centered on an opaque `LogSender` in `src/logging/mod.rs`.

Facts from the current code:

- `LogSender` is the only outward-facing application logging handle used by non-logging code.
- `LogSender` exposes `send(event)` where `event` implements the sealed `DomainLogEvent` trait.
- `LogSender::send(...)` only returns `LogSendError::QueueClosed`.
- `LogSender::send(...)` does not expose field bags, records, severities, tracing APIs, or queue internals.
- `LogSender` filters by minimum app severity before queueing, using event metadata severity.
- `LogSender` eagerly materializes the typed event into the private `raw_record::QueuedRecord` shape before queueing.
- The queue payload type is private to `src/logging`.
- `LogWorker` receives queued records, converts them into final `LogRecord` values, and forwards them to the backend.
- `LogWorker` discards backend sink failures internally after dequeue. The worker currently does `let _ = self.backend.emit(&materialized);`.
- This means logging is best effort after enqueue.

Facts about the logging trait and domain ownership:

- `src/logging/event.rs` defines the sealed logging contract with `DomainLogEvent`, `SealedLogEvent`, `LogEventMetadata`, `LogEventSource`, `LogEventResult`, and `LogFieldVisitor`.
- Each domain owns its own typed log ADTs instead of routing application meaning through one central logging-owned sum enum.
- Runtime-owned events live in `src/runtime/log_event.rs`.
- DCS-owned events live in `src/dcs/log_event.rs`.
- PgInfo-owned events live in `src/pginfo/log_event.rs`.
- Process-owned events live in `src/process/log_event.rs`.
- Logging-internal postgres ingest events live in `src/logging/postgres_ingest.rs` as private typed enums.

Facts about process-domain logging after the refactor:

- `src/process/worker.rs` no longer has `emit_process_event(...)` wrappers.
- `src/process/worker.rs` now constructs `ProcessLogEvent` or `SubprocessLogEvent` values directly and calls `ctx.runtime.log.send(...)` directly.
- `src/process/log_event.rs` owns the process event taxonomy.
- `ProcessLogEvent` covers worker startup, request receipt, inbox disconnect, busy rejection, preflight failures, command-build failures, spawn failures, job started, timeout, exit success, exit failure, poll failure, output drain failure, and output emit failure.
- `SubprocessLogEvent` represents stdout/stderr lines from child processes as a separate typed event.
- `SubprocessLogEvent` carries producer, origin, execution identity, stream, and bytes.
- Process execution identity is modeled with `ProcessExecutionIdentity`, which embeds `ProcessJobIdentity`.
- Process job kind is recorded through `ProcessJobKind`.
- Process stdout lines map to info severity and child-stdout transport.
- Process stderr lines map to warn severity and child-stderr transport.
- Process log fields include `job.id`, `job.kind`, `binary`, `stream`, `bytes_len`, and `error` where appropriate.

Facts about process worker behavior and control flow after the refactor:

- Process worker log sends still map queue-closed errors into `WorkerError::Message(...)` at the point of send.
- Backend sink failures no longer affect the process worker after the event has been accepted by the queue.
- Output-drain and output-emit logging still publish typed process events, but only queue failure is visible to the caller.
- The process worker still transitions jobs back to idle and publishes outcomes after successful sends.
- Subprocess output capture is still controlled by `logging.capture_subprocess_output` in runtime configuration.
- `ProcessRuntime` stores `log: LogSender`, `capture_subprocess_output`, and the process command runner.

Facts about runtime startup and the log worker:

- `src/runtime/node.rs` bootstraps logging first.
- `bootstrap(...)` returns `LoggingSystem { sender, worker }`.
- Runtime sends the startup event through `log.send(runtime_startup_event(...))`.
- Worker orchestration now joins the non-fallible log worker separately from fallible workers using `tokio::join!`.
- Runtime, pginfo, dcs, process, HA, API, and postgres ingest all share cloned `LogSender` values.

Facts about postgres ingest in the new architecture:

- `src/logging/postgres_ingest.rs` no longer uses `emit_ingest_event(...)`, `emit_ingest_step_failure(...)`, `emit_ingest_retry_recovered(...)`, or `emit_postgres_line(...)`.
- Postgres ingest now constructs `PostgresIngestLogEvent` or `PostgresLineLogEvent` values directly and sends them through `LogSender`.
- `PostgresIngestLogEvent` covers step failure, recovery, and iteration summary.
- `PostgresLineLogEvent` covers JSON, plain, and unparsed postgres lines.
- The helper `postgres_line_event(...)` is a pure builder that returns a typed event. The caller performs the send.
- Postgres ingest queue-send failures are still surfaced as worker errors because they mean the logging queue is broken.
- Sink and backend failures after enqueue remain internal to logging.

Facts about tracing visibility:

- `tracing` usage remains inside `src/logging/mod.rs`.
- Non-logging modules do not use `tracing`.
- The private backend bridge currently uses `TracingBackend` and a private `dispatch_tracing_record_event(...)` helper.

Facts about DCS logging after the refactor:

- DCS owns its own typed events in `src/dcs/log_event.rs`.
- The refactor also corrected a misleading event mapping by using generic failure events `ConnectedStepFailed` and `InitialConnectFailed`.
- Those names now match the real failure boundary instead of implying that every connected failure was specifically a watch-refresh failure or that every initial connect failure was specifically a snapshot-read failure.

Facts about documentation impact:

- The existing `docs/src/explanation/process-management.md` page already discusses subprocess output capture and typed subprocess events.
- The new code makes the logging boundary more explicit: process code only holds an opaque `LogSender`, owns typed process log ADTs locally, and does not know about record rendering or backend sinks.
- A documentation update should stay within process-management scope and explain how process execution and logging interact today.
