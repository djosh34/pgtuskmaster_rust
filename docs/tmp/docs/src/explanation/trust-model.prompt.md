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

docs/src/explanation/trust-model.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/explanation/ha-decision-engine.md
docs/src/explanation/introduction.md
docs/src/explanation/overview.md
docs/src/explanation/process-management.md
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
src/pginfo/mod.rs
src/pginfo/query.rs
src/pginfo/startup.rs
src/pginfo/state.rs
src/pginfo/worker.rs
src/postgres_managed.rs
src/postgres_managed_conf.rs
src/postgres_roles.rs
src/process/jobs.rs
src/process/mod.rs
src/process/postmaster.rs
src/process/source.rs
src/process/startup.rs
src/process/state.rs
src/process/worker.rs
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
docs/tmp/docs/src/explanation/trust-model.prompt.md
docs/tmp/verbose_extra_context/managed-postgres-roles.md
docs/tmp/verbose_extra_context/trust-model.md


===== src/dcs/state.rs =====
use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::{DcsClientConfig, DcsEndpoint},
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness},
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, StatePublisher, StateSubscriber,
        SwitchoverTarget, SystemIdentifier, TimelineId,
    },
};

use super::command::DcsCommandInbox;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcsView {
    NotTrusted(NotTrustedView),
    Degraded(ClusterView),
    Coordinated(ClusterView),
}

impl DcsView {
    pub fn mode(&self) -> DcsMode {
        match self {
            Self::NotTrusted(_) => DcsMode::NotTrusted,
            Self::Degraded(_) => DcsMode::Degraded,
            Self::Coordinated(_) => DcsMode::Coordinated,
        }
    }

    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::NotTrusted(view) => view.observed_leadership(),
            Self::Degraded(view) | Self::Coordinated(view) => view.leadership().held(),
        }
    }

    pub fn cluster(&self) -> Option<&ClusterView> {
        match self {
            Self::NotTrusted(view) => Some(view.cluster()),
            Self::Degraded(view) | Self::Coordinated(view) => Some(view),
        }
    }

    pub fn is_coordinated(&self) -> bool {
        matches!(self, Self::Coordinated(_))
    }

    pub(crate) fn starting() -> Self {
        Self::NotTrusted(NotTrustedView {
            observed_leadership: None,
            cluster: ClusterView {
                members: BTreeMap::new(),
                leadership: LeadershipObservation::Open,
                switchover: SwitchoverView::None,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotTrustedView {
    observed_leadership: Option<LeaseEpoch>,
    cluster: ClusterView,
}

impl NotTrustedView {
    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        self.observed_leadership.as_ref()
    }

    pub fn cluster(&self) -> &ClusterView {
        &self.cluster
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterView {
    members: BTreeMap<MemberId, ClusterMemberView>,
    leadership: LeadershipObservation,
    switchover: SwitchoverView,
}

impl ClusterView {
    pub fn members(&self) -> impl Iterator<Item = (&MemberId, &ClusterMemberView)> {
        self.members.iter()
    }

    pub fn member_ids(&self) -> impl Iterator<Item = &MemberId> {
        self.members.keys()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn member(&self, member_id: &MemberId) -> Option<&ClusterMemberView> {
        self.members.get(member_id)
    }

    pub fn leadership(&self) -> &LeadershipObservation {
        &self.leadership
    }

    pub fn switchover(&self) -> &SwitchoverView {
        &self.switchover
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    #[allow(dead_code)]
    pub(crate) fn new(
        members: BTreeMap<MemberId, ClusterMemberView>,
        leadership: LeadershipObservation,
        switchover: SwitchoverView,
    ) -> Self {
        Self {
            members,
            leadership,
            switchover,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberView {
    postgres: MemberPostgresView,
    postgres_target: PgTcpTarget,
}

impl ClusterMemberView {
    pub fn postgres_target(&self) -> &PgTcpTarget {
        &self.postgres_target
    }

    pub fn postgres(&self) -> &MemberPostgresView {
        &self.postgres
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    #[allow(dead_code)]
    pub(crate) fn new(postgres: MemberPostgresView, postgres_target: PgTcpTarget) -> Self {
        Self {
            postgres,
            postgres_target,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberPostgresView {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

impl MemberPostgresView {
    pub fn readiness(&self) -> Readiness {
        match self {
            Self::Unknown { readiness, .. }
            | Self::Primary { readiness, .. }
            | Self::Replica { readiness, .. } => readiness.clone(),
        }
    }

    pub fn system_identifier(&self) -> Option<SystemIdentifier> {
        match self {
            Self::Unknown {
                system_identifier, ..
            }
            | Self::Primary {
                system_identifier, ..
            }
            | Self::Replica {
                system_identifier, ..
            } => *system_identifier,
        }
    }

    pub fn timeline(&self) -> Option<TimelineId> {
        match self {
            Self::Unknown { timeline, .. } => *timeline,
            Self::Primary { committed_wal, .. } => committed_wal.timeline,
            Self::Replica {
                replay_wal,
                follow_wal,
                ..
            } => replay_wal
                .as_ref()
                .map(|position| position.timeline)
                .or_else(|| follow_wal.as_ref().map(|position| position.timeline))
                .flatten(),
        }
    }

    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Primary { .. })
    }

    pub fn is_ready_replica(&self) -> bool {
        matches!(
            self,
            Self::Replica {
                readiness: Readiness::Ready,
                ..
            }
        )
    }

    pub fn is_ready_non_primary(&self) -> bool {
        matches!(
            self,
            Self::Unknown {
                readiness: Readiness::Ready,
                ..
            }
                | Self::Replica {
                    readiness: Readiness::Ready,
                    ..
                }
        )
    }

    pub fn committed_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Primary { committed_wal, .. } => Some(committed_wal),
            Self::Unknown { .. } | Self::Replica { .. } => None,
        }
    }

    pub fn replay_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Replica { replay_wal, .. } => replay_wal.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub fn follow_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Replica { follow_wal, .. } => follow_wal.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub fn upstream(&self) -> Option<&MemberId> {
        match self {
            Self::Replica { upstream, .. } => upstream.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipObservation {
    Open,
    Held(LeaseEpoch),
}

impl LeadershipObservation {
    pub fn held(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::Open => None,
            Self::Held(epoch) => Some(epoch),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "target")]
pub enum SwitchoverView {
    None,
    Requested(SwitchoverTarget),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsEtcdConfig {
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) identity: DcsNodeIdentity,
    pub(crate) etcd: DcsEtcdConfig,
    pub(crate) cadence: DcsCadence,
    pub(crate) advertisement: DcsLocalMemberAdvertisement,
    pub(crate) observed: DcsObservedState,
    pub(crate) state_channel: DcsStateChannel,
    pub(crate) control: DcsControlPlane,
    pub(crate) runtime: DcsRuntime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsNodeIdentity {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCadence {
    pub(crate) poll_interval: Duration,
    pub(crate) member_ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsLocalMemberAdvertisement {
    pub(crate) postgres: PgTcpTarget,
}

#[derive(Clone, Debug)]
pub(crate) struct DcsObservedState {
    pub(crate) pg: StateSubscriber<PgInfoState>,
}

pub(crate) struct DcsStateChannel {
    pub(crate) publisher: StatePublisher<DcsView>,
    pub(crate) cache: DcsCache,
}

impl DcsStateChannel {
    pub(crate) fn new(publisher: StatePublisher<DcsView>) -> Self {
        Self {
            publisher,
            cache: DcsCache {
                member_records: BTreeMap::new(),
                leader_record: None,
                switchover_record: None,
            },
        }
    }
}

pub(crate) struct DcsControlPlane {
    pub(crate) command_inbox: DcsCommandInbox,
}

pub(crate) struct DcsRuntime {
    pub(crate) log: LogHandle,
    pub(crate) last_emitted_mode: Option<DcsMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberLeaseRecord {
    pub(crate) owner: MemberId,
    pub(crate) ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) lease: MemberLeaseRecord,
    pub(crate) postgres_target: PgTcpTarget,
    pub(crate) postgres: MemberPostgresRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum MemberPostgresRecord {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeadershipRecord {
    pub(crate) epoch: LeaseEpoch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SwitchoverRecord {
    pub(crate) target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DcsCache {
    pub(crate) member_records: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader_record: Option<LeadershipRecord>,
    pub(crate) switchover_record: Option<SwitchoverRecord>,
}

pub(crate) fn evaluate_mode(etcd_reachable: bool, cache: &DcsCache, self_id: &MemberId) -> DcsMode {
    if !etcd_reachable {
        return DcsMode::NotTrusted;
    }

    if !cache.member_records.contains_key(self_id) {
        return DcsMode::Degraded;
    }

    if !has_member_quorum(cache) {
        return DcsMode::Degraded;
    }

    DcsMode::Coordinated
}

fn has_member_quorum(cache: &DcsCache) -> bool {
    if cache.member_records.len() <= 1 {
        cache.member_records.len() == 1
    } else {
        cache.member_records.len() >= 2
    }
}

pub(crate) fn build_dcs_view(mode: DcsMode, cache: &DcsCache) -> DcsView {
    let authoritative_leader = cache
        .leader_record
        .as_ref()
        .map(|record| record.epoch.holder.clone());
    let cluster = ClusterView {
        members: cache
            .member_records
            .iter()
            .map(|(member_id, record)| {
                (
                    member_id.clone(),
                    build_member_view(member_id, record, authoritative_leader.as_ref()),
                )
            })
            .collect(),
        leadership: cache
            .leader_record
            .as_ref()
            .map(|record| LeadershipObservation::Held(record.epoch.clone()))
            .unwrap_or(LeadershipObservation::Open),
        switchover: cache
            .switchover_record
            .as_ref()
            .map(|record| SwitchoverView::Requested(record.target.clone()))
            .unwrap_or(SwitchoverView::None),
    };

    match mode {
        DcsMode::NotTrusted => DcsView::NotTrusted(NotTrustedView {
            observed_leadership: cache.leader_record.as_ref().map(|record| record.epoch.clone()),
            cluster,
        }),
        DcsMode::Degraded => DcsView::Degraded(cluster),
        DcsMode::Coordinated => DcsView::Coordinated(cluster),
    }
}

fn build_member_view(
    member_id: &MemberId,
    record: &MemberRecord,
    authoritative_leader: Option<&MemberId>,
) -> ClusterMemberView {
    ClusterMemberView {
        postgres: match &record.postgres {
            MemberPostgresRecord::Unknown {
                readiness,
                timeline,
                system_identifier,
            } => MemberPostgresView::Unknown {
                readiness: readiness.clone(),
                timeline: *timeline,
                system_identifier: *system_identifier,
            },
            MemberPostgresRecord::Primary {
                readiness,
                system_identifier,
                committed_wal,
            } => {
                if authoritative_leader.is_some_and(|leader| leader != member_id) {
                    MemberPostgresView::Unknown {
                        readiness: readiness.clone(),
                        timeline: committed_wal.timeline,
                        system_identifier: *system_identifier,
                    }
                } else {
                    MemberPostgresView::Primary {
                        readiness: readiness.clone(),
                        system_identifier: *system_identifier,
                        committed_wal: committed_wal.clone(),
                    }
                }
            }
            MemberPostgresRecord::Replica {
                readiness,
                system_identifier,
                upstream,
                replay_wal,
                follow_wal,
            } => MemberPostgresView::Replica {
                readiness: readiness.clone(),
                system_identifier: *system_identifier,
                upstream: upstream.clone(),
                replay_wal: replay_wal.clone(),
                follow_wal: follow_wal.clone(),
            },
        },
        postgres_target: record.postgres_target.clone(),
    }
}

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_target: &PgTcpTarget,
    lease_ttl_ms: u64,
    pg_state: &PgInfoState,
    previous_record: Option<&MemberRecord>,
) -> MemberRecord {
    let lease = MemberLeaseRecord {
        owner: self_id.clone(),
        ttl_ms: lease_ttl_ms,
    };

    let postgres = match pg_state {
        PgInfoState::Unknown { common } => MemberPostgresRecord::Unknown {
            readiness: common.readiness.clone(),
            timeline: common
                .timeline
                .or_else(|| previous_record.and_then(member_record_timeline)),
            system_identifier: common
                .system_identifier
                .or_else(|| previous_record.and_then(member_record_system_identifier)),
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberPostgresRecord::Primary {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            committed_wal: ObservedWalPosition {
                timeline: common.timeline,
                lsn: *wal_lsn,
            },
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => MemberPostgresRecord::Replica {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            upstream: upstream.as_ref().map(|value| value.member_id.clone()),
            replay_wal: Some(ObservedWalPosition {
                timeline: common.timeline,
                lsn: *replay_lsn,
            }),
            follow_wal: follow_lsn.map(|lsn| ObservedWalPosition {
                timeline: common.timeline,
                lsn,
            }),
        },
    };

    MemberRecord {
        lease,
        postgres_target: postgres_target.clone(),
        postgres,
    }
}

fn member_record_timeline(record: &MemberRecord) -> Option<TimelineId> {
    match &record.postgres {
        MemberPostgresRecord::Unknown { timeline, .. } => *timeline,
        MemberPostgresRecord::Primary { committed_wal, .. } => committed_wal.timeline,
        MemberPostgresRecord::Replica {
            replay_wal,
            follow_wal,
            ..
        } => replay_wal
            .as_ref()
            .and_then(|value| value.timeline)
            .or_else(|| follow_wal.as_ref().and_then(|value| value.timeline)),
    }
}

fn member_record_system_identifier(record: &MemberRecord) -> Option<SystemIdentifier> {
    match &record.postgres {
        MemberPostgresRecord::Unknown {
            system_identifier, ..
        }
        | MemberPostgresRecord::Primary {
            system_identifier, ..
        }
        | MemberPostgresRecord::Replica {
            system_identifier, ..
        } => *system_identifier,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        pginfo::state::{PgInfoState, Readiness},
        state::{LeaseEpoch, MemberId, PgTcpTarget, SystemIdentifier, TimelineId, WalLsn},
    };

    use super::{
        build_dcs_view, build_local_member_record, DcsCache, DcsMode, LeadershipObservation,
        LeadershipRecord,
        MemberLeaseRecord, MemberPostgresRecord, MemberPostgresView, MemberRecord,
        ObservedWalPosition,
    };

    fn member_record(postgres: MemberPostgresRecord) -> Result<MemberRecord, String> {
        Ok(MemberRecord {
            lease: MemberLeaseRecord {
                owner: MemberId("owner".to_string()),
                ttl_ms: 5_000,
            },
            postgres_target: PgTcpTarget::new("127.0.0.1".to_string(), 5432)?,
            postgres,
        })
    }

    #[test]
    fn build_dcs_view_hides_non_leader_primary_records() -> Result<(), String> {
        let mut member_records = BTreeMap::new();
        member_records.insert(
            MemberId("node-a".to_string()),
            member_record(MemberPostgresRecord::Primary {
                readiness: Readiness::Ready,
                system_identifier: None,
                committed_wal: ObservedWalPosition {
                    timeline: None,
                    lsn: WalLsn(42),
                },
            })?,
        );
        member_records.insert(
            MemberId("node-b".to_string()),
            member_record(MemberPostgresRecord::Primary {
                readiness: Readiness::Ready,
                system_identifier: None,
                committed_wal: ObservedWalPosition {
                    timeline: None,
                    lsn: WalLsn(41),
                },
            })?,
        );
        let cache = DcsCache {
            member_records,
            leader_record: Some(LeadershipRecord {
                epoch: LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 7,
                },
            }),
            switchover_record: None,
        };

        let cluster = match build_dcs_view(DcsMode::Coordinated, &cache) {
            super::DcsView::Coordinated(cluster) => cluster,
            other => return Err(format!("expected coordinated view, got {other:?}")),
        };

        if cluster.leadership()
            != &LeadershipObservation::Held(LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            })
        {
            return Err("expected node-a leadership to remain authoritative".to_string());
        }

        match cluster
            .member(&MemberId("node-a".to_string()))
            .ok_or_else(|| "missing node-a member".to_string())?
            .postgres()
        {
            MemberPostgresView::Primary { .. } => {}
            other => return Err(format!("expected node-a to remain primary, got {other:?}")),
        }

        match cluster
            .member(&MemberId("node-b".to_string()))
            .ok_or_else(|| "missing node-b member".to_string())?
            .postgres()
        {
            MemberPostgresView::Unknown { readiness, .. } if readiness == &Readiness::Ready => {}
            other => {
                return Err(format!(
                    "expected stale non-leader primary to be downgraded, got {other:?}"
                ))
            }
        }

        Ok(())
    }

    #[test]
    fn build_local_member_record_preserves_last_known_identity_when_pg_is_unknown() -> Result<(), String> {
        let previous = member_record(MemberPostgresRecord::Primary {
            readiness: Readiness::Ready,
            system_identifier: Some(SystemIdentifier(41)),
            committed_wal: ObservedWalPosition {
                timeline: Some(TimelineId(7)),
                lsn: WalLsn(42),
            },
        })?;
        let pg_state = PgInfoState::Unknown {
            common: crate::pginfo::state::PgInfoCommon {
                worker: crate::state::WorkerStatus::Running,
                sql: crate::pginfo::state::SqlStatus::Unreachable,
                readiness: Readiness::NotReady,
                timeline: None,
                system_identifier: None,
                pg_config: crate::pginfo::state::PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        };

        let record = build_local_member_record(
            &MemberId("node-a".to_string()),
            &PgTcpTarget::new("127.0.0.1".to_string(), 5432)?,
            5_000,
            &pg_state,
            Some(&previous),
        );

        match record.postgres {
            MemberPostgresRecord::Unknown {
                timeline,
                system_identifier,
                ..
            } => {
                if timeline != Some(TimelineId(7)) {
                    return Err(format!("expected preserved timeline, got {timeline:?}"));
                }
                if system_identifier != Some(SystemIdentifier(41)) {
                    return Err(format!(
                        "expected preserved system identifier, got {system_identifier:?}"
                    ));
                }
            }
            other => return Err(format!("expected unknown member record, got {other:?}")),
        }

        Ok(())
    }
}


===== src/ha/decide.rs =====
use std::cmp::Ordering;

use crate::{dcs::DcsMode, state::MemberId};

use super::types::{
    ApiVisibility, AuthorityProjection, Candidacy, DesiredState, ElectionEligibility, FailSafeGoal,
    FailureRecovery, FenceCutoff, FenceReason, FollowGoal, IdleReason, LeadershipView,
    LocalDataState, NoPrimaryFence, NoPrimaryProjection, PeerKnowledge, PeerLeaderState,
    PostgresState, ProcessState, PublicationGoal, PublicationState, RecoveryPlan, StorageState,
    SwitchoverState, TargetRole, WalPosition, WorldView,
};
use crate::state::{LeaseEpoch, SwitchoverTarget};

pub(crate) fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    if world.global.coordination.mode != DcsMode::Coordinated {
        return decide_degraded(world);
    }

    if world.local.storage == StorageState::Stalled {
        if let PostgresState::Primary { committed_lsn } = &world.local.postgres {
            let fence = active_or_observed_epoch(world).map(|epoch| FenceCutoff {
                epoch,
                committed_lsn: *committed_lsn,
            });
            return DesiredState {
                role: TargetRole::Fenced(FenceReason::StorageStalled),
                publication: no_primary_publication(NoPrimaryProjection::Recovering {
                    epoch: active_or_observed_epoch(world),
                    fence: fence
                        .map(NoPrimaryFence::Cutoff)
                        .unwrap_or(NoPrimaryFence::None),
                }),
                clear_switchover: false,
            };
        }
    }

    match &world.global.coordination.leadership {
        LeadershipView::HeldBySelf(epoch) => decide_as_lease_holder(world, self_id, epoch.clone()),
        LeadershipView::HeldByPeer { epoch, state } => {
            decide_under_foreign_leadership(world, epoch.clone(), state)
        }
        LeadershipView::Open | LeadershipView::StaleObservedLease { .. } => {
            decide_without_lease(world, self_id)
        }
    }
}

fn decide_degraded(world: &WorldView) -> DesiredState {
    match &world.local.postgres {
        PostgresState::Primary { committed_lsn } => {
            if let Some(epoch) = active_or_observed_epoch(world) {
                let cutoff = FenceCutoff {
                    epoch,
                    committed_lsn: *committed_lsn,
                };
                return DesiredState {
                    role: TargetRole::FailSafe(FailSafeGoal::PrimaryMustStop(cutoff.clone())),
                    publication: no_primary_publication(NoPrimaryProjection::DcsDegraded {
                        fence: NoPrimaryFence::Cutoff(cutoff),
                    }),
                    clear_switchover: false,
                };
            }

            DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
                publication: no_primary_publication(NoPrimaryProjection::DcsDegraded {
                    fence: NoPrimaryFence::None,
                }),
                clear_switchover: false,
            }
        }
        PostgresState::Replica { upstream, .. } => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(upstream.clone())),
            publication: no_primary_publication(NoPrimaryProjection::DcsDegraded {
                fence: NoPrimaryFence::None,
            }),
            clear_switchover: false,
        },
        PostgresState::Offline => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: no_primary_publication(NoPrimaryProjection::DcsDegraded {
                fence: NoPrimaryFence::None,
            }),
            clear_switchover: false,
        },
    }
}

fn decide_under_foreign_leadership(
    world: &WorldView,
    epoch: LeaseEpoch,
    state: &PeerLeaderState,
) -> DesiredState {
    let publication = match state {
        PeerLeaderState::PrimaryReady => primary_publication(epoch.clone()),
        PeerLeaderState::Recovering | PeerLeaderState::Unreachable => {
            no_primary_publication(NoPrimaryProjection::Recovering {
                epoch: Some(epoch.clone()),
                fence: NoPrimaryFence::None,
            })
        }
    };

    match (&world.local.postgres, state) {
        (PostgresState::Primary { .. }, _) => DesiredState {
            role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
            publication,
            clear_switchover: false,
        },
        (PostgresState::Offline | PostgresState::Replica { .. }, PeerLeaderState::PrimaryReady) => {
            DesiredState {
                role: TargetRole::Follower(follow_goal(world, epoch.holder)),
                publication,
                clear_switchover: false,
            }
        }
        (PostgresState::Offline | PostgresState::Replica { .. }, _) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication,
            clear_switchover: false,
        },
    }
}

fn decide_as_lease_holder(
    world: &WorldView,
    self_id: &MemberId,
    epoch: LeaseEpoch,
) -> DesiredState {
    let publication = leader_publication(world, self_id, &epoch);
    if !matches!(world.local.postgres, PostgresState::Primary { .. })
        && matches!(
            world.global.switchover,
            SwitchoverState::Requested(super::types::SwitchoverRequest {
                target: SwitchoverTarget::AnyHealthyReplica,
            })
        )
    {
        return DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: true,
        };
    }
    let allow_self_switchover_target =
        !matches!(world.local.postgres, PostgresState::Primary { .. });

    match resolve_switchover(world, self_id, allow_self_switchover_target) {
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication,
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Leader(epoch.clone()),
            publication,
            clear_switchover: true,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::DemotingForSwitchover(target),
            publication: PublicationGoal::KeepCurrent,
            clear_switchover: false,
        },
        ResolvedSwitchover::Pending => DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: true,
        },
    }
}

fn decide_without_lease(world: &WorldView, self_id: &MemberId) -> DesiredState {
    match resolve_switchover(world, self_id, true) {
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Candidate(Candidacy::TargetedSwitchover(target)),
            publication: open_or_stale_publication(world),
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingTarget(target)),
            publication: open_or_stale_publication(world),
            clear_switchover: false,
        },
        ResolvedSwitchover::Pending => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: open_or_stale_publication(world),
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: open_or_stale_publication(world),
            clear_switchover: true,
        },
        ResolvedSwitchover::NotRequested
            if best_failover_candidate(&world.global.peers, &world.global.self_peer, self_id)
                == Some(self_id.clone()) =>
        {
            DesiredState {
                role: TargetRole::Candidate(candidacy_kind(world)),
                publication: open_or_stale_publication(world),
                clear_switchover: false,
            }
        }
        ResolvedSwitchover::NotRequested => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: open_or_stale_publication(world),
            clear_switchover: false,
        },
    }
}

fn leader_publication(
    world: &WorldView,
    self_id: &MemberId,
    epoch: &LeaseEpoch,
) -> PublicationGoal {
    match &world.local.postgres {
        PostgresState::Primary { .. } => primary_publication(epoch.clone()),
        PostgresState::Offline | PostgresState::Replica { .. } => {
            no_primary_publication(NoPrimaryProjection::Recovering {
                epoch: Some(LeaseEpoch {
                    holder: self_id.clone(),
                    generation: epoch.generation,
                }),
                fence: NoPrimaryFence::None,
            })
        }
    }
}

fn follow_goal(world: &WorldView, leader: MemberId) -> FollowGoal {
    let recovery = match &world.local.data_dir {
        super::types::DataDirState::Missing => RecoveryPlan::Basebackup,
        super::types::DataDirState::Initialized(LocalDataState::BootstrapEmpty) => {
            RecoveryPlan::Basebackup
        }
        super::types::DataDirState::Initialized(LocalDataState::ConsistentReplica) => {
            match &world.local.postgres {
                PostgresState::Replica { upstream, .. } if upstream.as_ref() == Some(&leader) => {
                    RecoveryPlan::None
                }
                PostgresState::Replica { .. }
                | PostgresState::Offline
                | PostgresState::Primary { .. } => {
                    if rewind_failed_and_requires_basebackup(&world.local.process) {
                        RecoveryPlan::Basebackup
                    } else {
                        RecoveryPlan::StartStreaming
                    }
                }
            }
        }
        super::types::DataDirState::Initialized(LocalDataState::Diverged(state)) => match state {
            super::types::DivergenceState::RewindPossible => {
                if rewind_failed_and_requires_basebackup(&world.local.process) {
                    RecoveryPlan::Basebackup
                } else if world.local.observation.basebackup_completed_awaiting_start() {
                    RecoveryPlan::StartStreaming
                } else {
                    RecoveryPlan::Rewind
                }
            }
            super::types::DivergenceState::BasebackupRequired => RecoveryPlan::Basebackup,
        },
    };

    FollowGoal { leader, recovery }
}

fn rewind_failed_and_requires_basebackup(process: &ProcessState) -> bool {
    matches!(
        process,
        ProcessState::Failed(super::types::JobFailure {
            job: crate::process::jobs::ActiveJobKind::PgRewind,
            recovery: FailureRecovery::FallbackToBasebackup,
        })
    )
}

fn candidacy_kind(world: &WorldView) -> Candidacy {
    match &world.local.data_dir {
        super::types::DataDirState::Missing
        | super::types::DataDirState::Initialized(LocalDataState::BootstrapEmpty) => {
            Candidacy::Bootstrap
        }
        _ => {
            if matches!(
                world.local.publication,
                PublicationState::Projected(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::DcsDegraded { .. }
                ))
            ) {
                Candidacy::ResumeAfterOutage
            } else {
                Candidacy::Failover
            }
        }
    }
}

fn active_or_observed_epoch(world: &WorldView) -> Option<LeaseEpoch> {
    match &world.global.coordination.leadership {
        LeadershipView::Open => None,
        LeadershipView::HeldBySelf(epoch)
        | LeadershipView::HeldByPeer { epoch, .. }
        | LeadershipView::StaleObservedLease { epoch, .. } => Some(epoch.clone()),
    }
}

fn primary_publication(epoch: LeaseEpoch) -> PublicationGoal {
    PublicationGoal::Publish(AuthorityProjection::Primary(epoch))
}

fn no_primary_publication(projection: NoPrimaryProjection) -> PublicationGoal {
    PublicationGoal::Publish(AuthorityProjection::NoPrimary(projection))
}

fn open_or_stale_publication(world: &WorldView) -> PublicationGoal {
    match &world.global.coordination.leadership {
        LeadershipView::Open => no_primary_publication(NoPrimaryProjection::LeaseOpen),
        LeadershipView::StaleObservedLease { epoch, reason } => {
            no_primary_publication(NoPrimaryProjection::StaleObservedLease {
                epoch: epoch.clone(),
                reason: reason.clone(),
            })
        }
        LeadershipView::HeldByPeer { epoch, .. } | LeadershipView::HeldBySelf(epoch) => {
            no_primary_publication(NoPrimaryProjection::Recovering {
                epoch: Some(epoch.clone()),
                fence: NoPrimaryFence::None,
            })
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedSwitchover {
    NotRequested,
    Proceed(MemberId),
    Pending,
    Abandon,
}

fn resolve_switchover(
    world: &WorldView,
    self_id: &MemberId,
    allow_self_target: bool,
) -> ResolvedSwitchover {
    match &world.global.switchover {
        SwitchoverState::None => ResolvedSwitchover::NotRequested,
        SwitchoverState::Requested(request) => match &request.target {
            SwitchoverTarget::AnyHealthyReplica => best_switchover_target(
                &world.global.peers,
                &world.global.self_peer,
                self_id,
                allow_self_target,
            )
            .map_or(ResolvedSwitchover::Pending, ResolvedSwitchover::Proceed),
            SwitchoverTarget::Specific(member_id) => {
                if member_id == self_id {
                    if allow_self_target && switchover_target_is_valid(&world.global.self_peer) {
                        ResolvedSwitchover::Proceed(member_id.clone())
                    } else {
                        ResolvedSwitchover::Abandon
                    }
                } else if world
                    .global
                    .peers
                    .get(member_id)
                    .is_some_and(switchover_target_is_valid)
                {
                    ResolvedSwitchover::Proceed(member_id.clone())
                } else {
                    ResolvedSwitchover::Abandon
                }
            }
        },
    }
}

fn best_switchover_target(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
    allow_self_target: bool,
) -> Option<MemberId> {
    if allow_self_target && switchover_target_is_valid(self_peer) {
        return Some(self_id.clone());
    }

    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| switchover_target_is_valid(peer))
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_switchover_candidates(left_id, left_peer, right_id, right_peer)
        })
        .map(|(member_id, _)| member_id);

    peer_candidate
}

fn best_failover_candidate(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
) -> Option<MemberId> {
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| classify_candidate(peer).is_some())
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_candidate_rank(
                candidate_rank(&left_peer.eligibility),
                left_id,
                candidate_rank(&right_peer.eligibility),
                right_id,
            )
        })
        .map(|(member_id, _)| member_id);

    if classify_candidate(self_peer).is_none() {
        return peer_candidate;
    }

    match peer_candidate {
        Some(peer_id) => {
            let peer_rank = peers
                .get(&peer_id)
                .map(|peer| candidate_rank(&peer.eligibility));
            if compare_candidate_rank(
                candidate_rank(&self_peer.eligibility),
                self_id,
                peer_rank.flatten(),
                &peer_id,
            ) == Ordering::Greater
            {
                Some(self_id.clone())
            } else {
                Some(peer_id)
            }
        }
        None => Some(self_id.clone()),
    }
}

fn switchover_target_is_valid(peer: &PeerKnowledge) -> bool {
    matches!(peer.api, ApiVisibility::Reachable)
        && matches!(peer.eligibility, ElectionEligibility::PromoteEligible(_))
}

fn compare_switchover_candidates(
    left_id: &MemberId,
    left_peer: &PeerKnowledge,
    right_id: &MemberId,
    right_peer: &PeerKnowledge,
) -> Ordering {
    compare_candidate_rank(
        candidate_rank(&left_peer.eligibility),
        left_id,
        candidate_rank(&right_peer.eligibility),
        right_id,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CandidateRank {
    Promote(WalPosition),
    Bootstrap,
}

fn candidate_rank(value: &ElectionEligibility) -> Option<CandidateRank> {
    match value {
        ElectionEligibility::PromoteEligible(position) => {
            Some(CandidateRank::Promote(position.clone()))
        }
        ElectionEligibility::BootstrapEligible => Some(CandidateRank::Bootstrap),
        ElectionEligibility::Ineligible(_) => None,
    }
}

fn compare_candidate_rank(
    left: Option<CandidateRank>,
    left_id: &MemberId,
    right: Option<CandidateRank>,
    right_id: &MemberId,
) -> Ordering {
    match (left, right) {
        (Some(CandidateRank::Promote(left_pos)), Some(CandidateRank::Promote(right_pos))) => {
            left_pos.cmp(&right_pos).then_with(|| right_id.cmp(left_id))
        }
        (Some(CandidateRank::Promote(_)), Some(CandidateRank::Bootstrap)) => Ordering::Greater,
        (Some(CandidateRank::Bootstrap), Some(CandidateRank::Promote(_))) => Ordering::Less,
        (Some(CandidateRank::Bootstrap), Some(CandidateRank::Bootstrap)) => right_id.cmp(left_id),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

fn classify_candidate(peer: &PeerKnowledge) -> Option<()> {
    match &peer.eligibility {
        ElectionEligibility::BootstrapEligible | ElectionEligibility::PromoteEligible(_) => {
            Some(())
        }
        ElectionEligibility::Ineligible(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{best_failover_candidate, decide};
    use crate::{
        dcs::DcsMode,
        state::{LeaseEpoch, MemberId, SwitchoverTarget, UnixMillis},
    };

    use super::super::types::{
        ApiVisibility, AuthorityProjection, Candidacy, CoordinationState, DataDirState,
        DesiredState, DivergenceState, ElectionEligibility, FollowGoal, GlobalKnowledge,
        IdleReason, IneligibleReason, LeadershipView, LocalDataState, LocalKnowledge,
        NoPrimaryFence, NoPrimaryProjection, ObservationState, ObservedPrimary, PeerKnowledge,
        PeerLeaderState, PostgresState, PrimaryObservation, ProcessState, PublicationGoal,
        PublicationState, RecoveryPlan, ReplicationState, StorageState, SwitchoverRequest,
        SwitchoverState, TargetRole, WalPosition, WorldView,
    };

    fn promote_peer(lsn: u64) -> PeerKnowledge {
        PeerKnowledge {
            eligibility: ElectionEligibility::PromoteEligible(WalPosition { timeline: 1, lsn }),
            api: ApiVisibility::Reachable,
        }
    }

    fn world(local: LocalKnowledge, self_peer: PeerKnowledge) -> WorldView {
        WorldView {
            local,
            global: GlobalKnowledge {
                coordination: CoordinationState {
                    mode: DcsMode::Coordinated,
                    leadership: LeadershipView::Open,
                    primary: PrimaryObservation::Absent,
                },
                switchover: SwitchoverState::None,
                peers: BTreeMap::new(),
                self_peer,
            },
        }
    }

    #[test]
    fn best_failover_candidate_includes_self_in_ranking() {
        let self_id = MemberId("node-a".to_string());
        let peers = BTreeMap::from([(MemberId("node-b".to_string()), promote_peer(10))]);

        assert_eq!(
            best_failover_candidate(&peers, &promote_peer(20), &self_id),
            Some(self_id)
        );
    }

    #[test]
    fn best_failover_candidate_prefers_higher_ranked_peer() {
        let self_id = MemberId("node-a".to_string());
        let peer_id = MemberId("node-b".to_string());
        let peers = BTreeMap::from([(peer_id.clone(), promote_peer(20))]);

        assert_eq!(
            best_failover_candidate(&peers, &promote_peer(10), &self_id),
            Some(peer_id)
        );
    }

    #[test]
    fn stale_observed_lease_does_not_block_failover_candidacy() {
        let self_id = MemberId("node-a".to_string());
        let stale_epoch = LeaseEpoch {
            holder: MemberId("node-b".to_string()),
            generation: 7,
        };
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Replica {
                    upstream: Some(MemberId("node-b".to_string())),
                    replication: ReplicationState::Streaming(WalPosition {
                        timeline: 1,
                        lsn: 42,
                    }),
                },
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::Projected(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::LeaseOpen,
                )),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            promote_peer(42),
        );
        world.global.coordination.leadership = LeadershipView::StaleObservedLease {
            epoch: stale_epoch.clone(),
            reason: super::super::types::StaleLeaseReason::HolderNotPrimary,
        };

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::Candidate(Candidacy::Failover),
                publication: PublicationGoal::Publish(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::StaleObservedLease {
                        epoch: stale_epoch,
                        reason: super::super::types::StaleLeaseReason::HolderNotPrimary,
                    },
                )),
                clear_switchover: false,
            }
        );
    }

    #[test]
    fn sampled_primary_without_lease_promotes_best_candidate() {
        let self_id = MemberId("node-a".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Offline,
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            promote_peer(42),
        );
        world.global.coordination.primary = PrimaryObservation::Observed(ObservedPrimary {
            member: MemberId("node-b".to_string()),
            timeline: None,
            system_identifier: None,
        });

        assert_eq!(
            decide(&world, &self_id).role,
            TargetRole::Candidate(Candidacy::Failover)
        );
    }

    #[test]
    fn basebackup_success_on_diverged_data_transitions_to_start_streaming() {
        let self_id = MemberId("node-b".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::Diverged(
                    DivergenceState::RewindPossible,
                )),
                postgres: PostgresState::Offline,
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(100),
                    last_start_success_at: Some(UnixMillis(10)),
                    last_basebackup_success_at: Some(UnixMillis(20)),
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            promote_peer(42),
        );
        world.global.coordination.leadership = LeadershipView::HeldByPeer {
            epoch: LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            },
            state: PeerLeaderState::PrimaryReady,
        };

        assert_eq!(
            decide(&world, &self_id).role,
            TargetRole::Follower(FollowGoal {
                leader: MemberId("node-a".to_string()),
                recovery: RecoveryPlan::StartStreaming,
            })
        );
    }

    #[test]
    fn idle_when_no_leader_no_candidate_and_no_switchover() {
        let self_id = MemberId("node-a".to_string());
        let world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Offline,
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            PeerKnowledge {
                eligibility: ElectionEligibility::Ineligible(IneligibleReason::StartingUp),
                api: ApiVisibility::Unreachable,
            },
        );

        assert_eq!(
            decide(&world, &self_id).role,
            TargetRole::Idle(IdleReason::AwaitingLeader)
        );
    }

    #[test]
    fn generic_switchover_request_waits_for_future_eligible_target() {
        let self_id = MemberId("node-a".to_string());
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Primary { committed_lsn: 42 },
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            promote_peer(42),
        );
        world.global.coordination.leadership = LeadershipView::HeldBySelf(LeaseEpoch {
            holder: self_id.clone(),
            generation: 7,
        });
        world.global.switchover = SwitchoverState::Requested(SwitchoverRequest {
            target: SwitchoverTarget::AnyHealthyReplica,
        });
        world.global.peers = BTreeMap::from([(
            MemberId("node-b".to_string()),
            PeerKnowledge {
                eligibility: ElectionEligibility::Ineligible(IneligibleReason::NotReady),
                api: ApiVisibility::Reachable,
            },
        )]);

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::Leader(LeaseEpoch {
                    holder: self_id.clone(),
                    generation: 7,
                }),
                publication: PublicationGoal::Publish(AuthorityProjection::Primary(LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 7,
                },)),
                clear_switchover: false,
            }
        );
    }

    #[test]
    fn lease_holder_replica_can_self_select_for_generic_switchover_after_winning_lease() {
        let self_id = MemberId("node-c".to_string());
        let epoch = LeaseEpoch {
            holder: self_id.clone(),
            generation: 7,
        };
        let mut world = world(
            LocalKnowledge {
                data_dir: DataDirState::Initialized(LocalDataState::ConsistentReplica),
                postgres: PostgresState::Replica {
                    upstream: None,
                    replication: ReplicationState::Streaming(WalPosition {
                        timeline: 1,
                        lsn: 50,
                    }),
                },
                process: ProcessState::Idle,
                storage: StorageState::Healthy,
                managed_roles_reconciled: false,
                publication: PublicationState::unknown(),
                observation: ObservationState {
                    pg_observed_at: UnixMillis(0),
                    last_start_success_at: None,
                    last_basebackup_success_at: None,
                    last_promote_success_at: None,
                    last_demote_success_at: None,
                },
            },
            promote_peer(50),
        );
        world.global.coordination.leadership = LeadershipView::HeldBySelf(epoch.clone());
        world.global.switchover = SwitchoverState::Requested(SwitchoverRequest {
            target: SwitchoverTarget::AnyHealthyReplica,
        });
        world.global.peers = BTreeMap::from([(MemberId("node-a".to_string()), promote_peer(40))]);

        assert_eq!(
            decide(&world, &self_id),
            DesiredState {
                role: TargetRole::Leader(epoch.clone()),
                publication: PublicationGoal::Publish(AuthorityProjection::NoPrimary(
                    NoPrimaryProjection::Recovering {
                        epoch: Some(epoch),
                        fence: NoPrimaryFence::None,
                    },
                )),
                clear_switchover: true,
            }
        );
    }
}


===== src/config/schema.rs =====
use std::{
    collections::BTreeMap,
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::{defaults, endpoint::DcsEndpoint};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum SecretSource {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
    Env { env: String },
}

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            Self::PathConfig { path } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            Self::Inline { .. } => f
                .debug_tuple("SecretSource")
                .field(&"<inline redacted>")
                .finish(),
            Self::Env { env } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("env({env})"))
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCertificateMode {
    Optional,
    Required,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub struct ClientCommonName(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfig {
    pub cert_chain: InlineOrPath,
    pub private_key: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientIdentityConfig {
    pub cert: InlineOrPath,
    pub key: SecretSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientAuthConfig {
    pub client_ca: InlineOrPath,
    pub client_certificate: ClientCertificateMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum TlsServerConfig {
    #[default]
    Disabled,
    Enabled {
        identity: TlsServerIdentityConfig,
        client_auth: Option<TlsClientAuthConfig>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "client_certificate", rename_all = "snake_case")]
pub enum ApiClientAuthConfig {
    #[default]
    Disabled,
    Optional {
        client_ca: InlineOrPath,
    },
    Required {
        client_ca: InlineOrPath,
        #[serde(default)]
        allowed_common_names: Vec<ClientCommonName>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiTlsConfig {
    pub identity: TlsServerIdentityConfig,
    #[serde(default)]
    pub client_auth: ApiClientAuthConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum ApiTransportConfig {
    #[default]
    Http,
    Https {
        tls: ApiTlsConfig,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    #[serde(default)]
    pub ha: HaConfig,
    #[serde(default)]
    pub process: ProcessConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub api: ApiConfig,
    pub pgtm: Option<PgtmConfig>,
    #[serde(default)]
    pub debug: DebugConfig,
}

impl RuntimeConfig {
    pub fn postgres_socket_dir(&self) -> PathBuf {
        self.postgres
            .paths
            .socket_dir
            .clone()
            .unwrap_or_else(|| self.process.working_root.join("socket"))
    }

    pub fn postgres_log_file(&self) -> PathBuf {
        self.postgres
            .paths
            .log_file
            .clone()
            .unwrap_or_else(|| self.process.working_root.join("logs/postgres.log"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: String,
    pub scope: String,
    pub member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub paths: PostgresPathsConfig,
    #[serde(default)]
    pub network: PostgresNetworkConfig,
    #[serde(default = "defaults::default_postgres_connect_timeout_s")]
    pub connect_timeout_s: u32,
    #[serde(default = "defaults::default_postgres_database")]
    pub local_database: String,
    #[serde(default)]
    pub rewind: PostgresRewindConfig,
    #[serde(default)]
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub access: PostgresAccessConfig,
    #[serde(default)]
    pub extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresPathsConfig {
    pub data_dir: PathBuf,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresNetworkConfig {
    #[serde(default = "defaults::default_postgres_listen_host")]
    pub listen_host: String,
    #[serde(default = "defaults::default_postgres_listen_port")]
    pub listen_port: u16,
    pub advertise_port: Option<u16>,
}

impl Default for PostgresNetworkConfig {
    fn default() -> Self {
        Self {
            listen_host: defaults::default_postgres_listen_host(),
            listen_port: defaults::default_postgres_listen_port(),
            advertise_port: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRewindConfig {
    #[serde(default = "defaults::default_postgres_database")]
    pub database: String,
    #[serde(default)]
    pub transport: PostgresClientTransportConfig,
}

impl Default for PostgresRewindConfig {
    fn default() -> Self {
        Self {
            database: defaults::default_postgres_database(),
            transport: PostgresClientTransportConfig::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresClientTransportConfig {
    #[serde(default = "defaults::default_pg_ssl_mode")]
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
    pub ca_cert: Option<InlineOrPath>,
}

impl Default for PostgresClientTransportConfig {
    fn default() -> Self {
        Self {
            ssl_mode: defaults::default_pg_ssl_mode(),
            ca_cert: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoleAuthConfig {
    Password { password: SecretSource },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub struct PostgresRoleName(pub String);

impl PostgresRoleName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub struct ManagedPostgresRoleKey(pub String);

impl ManagedPostgresRoleKey {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresRolePrivilege {
    Login,
    Replication,
    Superuser,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfig {
    pub username: PostgresRoleName,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MandatoryPostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraManagedPostgresRoleConfig {
    #[serde(flatten)]
    pub role: PostgresRoleConfig,
    #[serde(default = "default_extra_managed_postgres_role_privilege")]
    pub privilege: PostgresRolePrivilege,
    #[serde(default)]
    pub member_of: Vec<PostgresRoleName>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfig {
    pub mandatory: MandatoryPostgresRolesConfig,
    #[serde(default)]
    pub extra: BTreeMap<ManagedPostgresRoleKey, ExtraManagedPostgresRoleConfig>,
}

const fn default_extra_managed_postgres_role_privilege() -> PostgresRolePrivilege {
    PostgresRolePrivilege::Login
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresAccessConfig {
    pub hba: InlineOrPath,
    pub ident: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<DcsEndpoint>,
    #[serde(default)]
    pub client: DcsClientConfig,
    pub init: Option<DcsInitConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DcsClientConfig {
    #[serde(default)]
    pub auth: DcsAuthConfig,
    #[serde(default)]
    pub tls: DcsTlsConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DcsAuthConfig {
    #[default]
    Disabled,
    Basic {
        username: String,
        password: SecretSource,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum DcsTlsConfig {
    #[default]
    Disabled,
    Enabled {
        ca_cert: Option<InlineOrPath>,
        identity: Option<TlsClientIdentityConfig>,
        server_name: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsInitConfig {
    pub payload_json: String,
    pub write_on_bootstrap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaConfig {
    #[serde(default = "defaults::default_ha_loop_interval_ms")]
    pub loop_interval_ms: u64,
    #[serde(default = "defaults::default_ha_lease_ttl_ms")]
    pub lease_ttl_ms: u64,
}

impl Default for HaConfig {
    fn default() -> Self {
        Self {
            loop_interval_ms: defaults::default_ha_loop_interval_ms(),
            lease_ttl_ms: defaults::default_ha_lease_ttl_ms(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    #[serde(default)]
    pub timeouts: ProcessTimeoutsConfig,
    #[serde(default = "defaults::default_runtime_working_root")]
    pub working_root: PathBuf,
    #[serde(default)]
    pub binaries: BinaryResolutionConfig,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            timeouts: ProcessTimeoutsConfig::default(),
            working_root: defaults::default_runtime_working_root(),
            binaries: BinaryResolutionConfig::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessTimeoutsConfig {
    #[serde(default = "defaults::default_pg_rewind_timeout_ms")]
    pub pg_rewind_ms: u64,
    #[serde(default = "defaults::default_bootstrap_timeout_ms")]
    pub bootstrap_ms: u64,
    #[serde(default = "defaults::default_fencing_timeout_ms")]
    pub fencing_ms: u64,
}

impl Default for ProcessTimeoutsConfig {
    fn default() -> Self {
        Self {
            pg_rewind_ms: defaults::default_pg_rewind_timeout_ms(),
            bootstrap_ms: defaults::default_bootstrap_timeout_ms(),
            fencing_ms: defaults::default_fencing_timeout_ms(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BinaryResolutionConfig {
    #[serde(default)]
    pub overrides: BinaryPathOverrides,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathOverrides {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostgresBinaryName {
    Postgres,
    PgCtl,
    PgRewind,
    Initdb,
    PgBasebackup,
    Psql,
}

impl PostgresBinaryName {
    pub fn executable_name(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::PgCtl => "pg_ctl",
            Self::PgRewind => "pg_rewind",
            Self::Initdb => "initdb",
            Self::PgBasebackup => "pg_basebackup",
            Self::Psql => "psql",
        }
    }

    pub fn config_field(self) -> &'static str {
        match self {
            Self::Postgres => "process.binaries.overrides.postgres",
            Self::PgCtl => "process.binaries.overrides.pg_ctl",
            Self::PgRewind => "process.binaries.overrides.pg_rewind",
            Self::Initdb => "process.binaries.overrides.initdb",
            Self::PgBasebackup => "process.binaries.overrides.pg_basebackup",
            Self::Psql => "process.binaries.overrides.psql",
        }
    }

    fn override_path(self, overrides: &BinaryPathOverrides) -> Option<&PathBuf> {
        match self {
            Self::Postgres => overrides.postgres.as_ref(),
            Self::PgCtl => overrides.pg_ctl.as_ref(),
            Self::PgRewind => overrides.pg_rewind.as_ref(),
            Self::Initdb => overrides.initdb.as_ref(),
            Self::PgBasebackup => overrides.pg_basebackup.as_ref(),
            Self::Psql => overrides.psql.as_ref(),
        }
    }
}

impl BinaryResolutionConfig {
    pub fn resolve_binary_path(&self, binary: PostgresBinaryName) -> Result<PathBuf, String> {
        if let Some(path) = binary.override_path(&self.overrides) {
            if !path.is_file() {
                return Err(format!(
                    "`{}` points to a missing executable: {}",
                    binary.config_field(),
                    path.display()
                ));
            }
            return Ok(path.clone());
        }

        let executable = binary.executable_name();
        let mut searched = Vec::new();

        if let Some(path_env) = std::env::var_os("PATH") {
            for directory in std::env::split_paths(&path_env) {
                let candidate = directory.join(executable);
                if candidate.is_file() {
                    return Ok(candidate);
                }
                searched.push(candidate);
            }
        }

        for directory in conventional_postgres_bin_dirs() {
            let candidate = directory.join(executable);
            if candidate.is_file() {
                return Ok(candidate);
            }
            searched.push(candidate);
        }

        let preview = searched
            .iter()
            .take(6)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let detail = if preview.is_empty() {
            "no candidate paths were discovered".to_string()
        } else {
            format!("searched {preview}")
        };

        Err(format!(
            "unable to resolve `{executable}` via PATH or conventional PostgreSQL install locations; {detail}; set `{}` explicitly if autodiscovery fails",
            binary.config_field()
        ))
    }
}

fn conventional_postgres_bin_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    directories.extend(all_child_bin_dirs(Path::new("/usr/lib/postgresql")));
    directories.extend(prefixed_child_bin_dirs(Path::new("/usr"), "pgsql-"));
    directories.extend(prefixed_child_bin_dirs(
        Path::new("/opt/homebrew/opt"),
        "postgresql@",
    ));
    directories.extend(prefixed_child_bin_dirs(
        Path::new("/usr/local/opt"),
        "postgresql@",
    ));
    directories.push(PathBuf::from("/opt/homebrew/opt/libpq/bin"));
    directories.push(PathBuf::from("/usr/local/opt/libpq/bin"));
    directories
}

fn all_child_bin_dirs(root: &Path) -> Vec<PathBuf> {
    child_dirs_matching(root, |_| true)
        .into_iter()
        .map(|path| path.join("bin"))
        .collect()
}

fn prefixed_child_bin_dirs(root: &Path, prefix: &str) -> Vec<PathBuf> {
    child_dirs_matching(root, |name| name.starts_with(prefix))
        .into_iter()
        .map(|path| path.join("bin"))
        .collect()
}

fn child_dirs_matching<F>(root: &Path, predicate: F) -> Vec<PathBuf>
where
    F: Fn(&str) -> bool,
{
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    let mut directories = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() {
                return None;
            }
            let name = entry.file_name();
            let name = name.to_str()?;
            predicate(name).then(|| entry.path())
        })
        .collect::<Vec<_>>();
    directories.sort();
    directories
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default = "defaults::default_logging_capture_subprocess_output")]
    pub capture_subprocess_output: bool,
    #[serde(default)]
    pub postgres: PostgresLoggingConfig,
    #[serde(default)]
    pub sinks: LoggingSinksConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            capture_subprocess_output: defaults::default_logging_capture_subprocess_output(),
            postgres: PostgresLoggingConfig::default(),
            sinks: LoggingSinksConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresLoggingConfig {
    #[serde(default = "defaults::default_logging_postgres_enabled")]
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    #[serde(default = "defaults::default_logging_postgres_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub cleanup: LogCleanupConfig,
}

impl Default for PostgresLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_postgres_enabled(),
            pg_ctl_log_file: None,
            log_dir: None,
            poll_interval_ms: defaults::default_logging_postgres_poll_interval_ms(),
            cleanup: LogCleanupConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingSinksConfig {
    #[serde(default)]
    pub stderr: StderrSinkConfig,
    #[serde(default)]
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrSinkConfig {
    #[serde(default = "defaults::default_logging_sink_stderr_enabled")]
    pub enabled: bool,
}

impl Default for StderrSinkConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_sink_stderr_enabled(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSinkConfig {
    #[serde(default = "defaults::default_logging_sink_file_enabled")]
    pub enabled: bool,
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub mode: FileSinkMode,
}

impl Default for FileSinkConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_sink_file_enabled(),
            path: None,
            mode: FileSinkMode::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSinkMode {
    #[default]
    Append,
    Truncate,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCleanupConfig {
    #[serde(default = "defaults::default_logging_cleanup_enabled")]
    pub enabled: bool,
    #[serde(default = "defaults::default_logging_cleanup_max_files")]
    pub max_files: u64,
    #[serde(default = "defaults::default_logging_cleanup_max_age_seconds")]
    pub max_age_seconds: u64,
    #[serde(default = "defaults::default_logging_cleanup_protect_recent_seconds")]
    pub protect_recent_seconds: u64,
}

impl Default for LogCleanupConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_cleanup_enabled(),
            max_files: defaults::default_logging_cleanup_max_files(),
            max_age_seconds: defaults::default_logging_cleanup_max_age_seconds(),
            protect_recent_seconds: defaults::default_logging_cleanup_protect_recent_seconds(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    #[serde(default = "defaults::default_api_listen_addr")]
    pub listen_addr: SocketAddr,
    #[serde(default)]
    pub transport: ApiTransportConfig,
    #[serde(default)]
    pub auth: ApiAuthConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: defaults::default_api_listen_addr(),
            transport: ApiTransportConfig::default(),
            auth: ApiAuthConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAuthConfig {
    #[default]
    Disabled,
    RoleTokens(ApiRoleTokensConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ApiRoleTokensConfig {
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PgtmApiTransportExpectation {
    Http,
    Https,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmConfig {
    #[serde(default)]
    pub api: PgtmApiConfig,
    #[serde(default)]
    pub postgres: PgtmPostgresConfig,
    pub primary_target: Option<PgtmPrimaryTargetConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmApiConfig {
    pub base_url: Option<String>,
    pub advertised_url: Option<String>,
    pub expected_transport: Option<PgtmApiTransportExpectation>,
    #[serde(default)]
    pub auth: PgtmApiAuthConfig,
    #[serde(default)]
    pub tls: PgtmClientTlsConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PgtmApiAuthConfig {
    #[default]
    Disabled,
    RoleTokens {
        read_token: Option<SecretSource>,
        admin_token: Option<SecretSource>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmPostgresConfig {
    #[serde(default)]
    pub tls: PgtmClientTlsConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmClientTlsConfig {
    pub ca_cert: Option<InlineOrPath>,
    pub identity: Option<TlsClientIdentityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmPrimaryTargetConfig {
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    #[serde(default = "defaults::default_debug_enabled")]
    pub enabled: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_debug_enabled(),
        }
    }
}


===== tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature =====
Feature: ha_dcs_quorum_lost_enters_failsafe
  Scenario: losing DCS quorum removes the operator-visible primary and exposes fail-safe behavior
    Given the "three_node_plain" harness is running
    And I wait for exactly one stable primary as "initial_primary"
    When I stop a DCS quorum majority
    Then there is no operator-visible primary across running nodes
    And every running node reports fail_safe in debug output
    And there is no dual-primary evidence during the transition window
    When I restore DCS quorum
    Then I wait for exactly one stable primary as "restored_primary"


===== docs/tmp/verbose_extra_context/trust-model.md =====
# Verbose Extra Context For `docs/src/explanation/trust-model.md`

This note exists to give K2 exhaustive raw context about the current trust model after the DCS rewrite. It is intentionally factual and implementation-centered.

## Current naming and public model

The old `FullQuorum` naming is no longer the current implementation. The public DCS coordination surface is now:

- `DcsView::NotTrusted(NotTrustedView)`
- `DcsView::Degraded(ClusterView)`
- `DcsView::Coordinated(ClusterView)`

The corresponding mode enum is:

- `DcsMode::NotTrusted`
- `DcsMode::Degraded`
- `DcsMode::Coordinated`

This matters for docs wording. The current code does not model "full quorum mathematics". The implementation uses simpler coordination-mode gates that are driven by etcd reachability, self visibility in the DCS member set, and a minimal member-count rule.

## Exact trust / coordination evaluation logic

The core logic is in `src/dcs/state.rs` in `evaluate_mode(etcd_reachable, cache, self_id)`.

The current decision order is:

1. If etcd is not reachable, return `DcsMode::NotTrusted`.
2. If the local node does not see its own member record in the DCS cache, return `DcsMode::Degraded`.
3. If the cache fails the member-count rule, return `DcsMode::Degraded`.
4. Otherwise return `DcsMode::Coordinated`.

So the current model is intentionally conservative but also intentionally simple:

- `NotTrusted` means the DCS transport itself is not currently trusted.
- `Degraded` means etcd is reachable, but the node does not have enough confidence in the cluster view to treat it as fully coordinated.
- `Coordinated` means etcd is reachable, the local member record is present, and the minimal member rule passes.

## Exact member-count rule

The member-count rule is implemented by `has_member_quorum(cache)` in `src/dcs/state.rs`.

The exact rule is:

- if there is 1 or fewer visible members, only exactly 1 member counts as coordinated
- if there are 2 or more visible members, at least 2 members are required for coordinated mode

This is not majority quorum math. It is a small safety rule that distinguishes:

- single-node deployments, where one visible member is enough
- multi-member deployments, where seeing only one member is treated as degraded

The docs should explain that clearly and should not imply the system is calculating a mathematical majority across arbitrary cluster sizes in this code path.

## What "fresh" means in practice

The raw config knobs are in `src/config/schema.rs`:

- `ha.loop_interval_ms`
- `ha.lease_ttl_ms`

The current defaults are defined in `src/config/defaults.rs`:

- `DEFAULT_HA_LOOP_INTERVAL_MS = 1_000`
- `DEFAULT_HA_LEASE_TTL_MS = 10_000`

These values are used as follows:

- `runtime/node.rs` converts `cfg.ha.loop_interval_ms` into the worker poll interval.
- `runtime/node.rs` passes `cfg.ha.lease_ttl_ms` into DCS as `DcsCadence.member_ttl_ms`.
- `dcs/worker.rs` uses `member_ttl_ms` as the staleness cutoff for the local PostgreSQL observation during member publication.

The exact local publication freshness check in `dcs/worker.rs` is:

- read the latest PostgreSQL observation timestamp with `pg_snapshot.last_refresh_at()`
- compare `now - last_refresh_at` against `member_ttl_ms`
- if that age is greater than `member_ttl_ms`, the local member entry is deleted from etcd and any locally held leadership lease is released

That means "fresh" is operationally defined as:

- the node has a recent PostgreSQL observation
- the age of that observation is not older than `ha.lease_ttl_ms`

When the observation is older than that threshold, the node stops advertising itself as a valid DCS member.

## Why self-member visibility matters

`evaluate_mode()` explicitly requires that the local node can still see its own member record in the DCS cache. This gives an important safety property:

- if the node can talk to etcd but does not see itself in the authoritative member set, it does not treat the cluster as fully coordinated

This is separate from transport reachability:

- etcd reachable but self missing => `Degraded`
- etcd unreachable => `NotTrusted`

That distinction is important for explaining why the node may still have some cluster information visible while refusing to act with full coordination authority.

## What `NotTrusted` still exposes

Although `NotTrusted` is the most conservative mode, the current implementation still retains the last observed cluster snapshot internally and exposes it through `DcsView::cluster()`. It also preserves the last observed leadership through `NotTrustedView`.

That behavior exists so the system can keep publishing conservative operator-visible information and fail-safe fencing context during DCS outages instead of collapsing the view to an empty cluster.

Docs should describe this carefully:

- `NotTrusted` does not mean "no information exists"
- it means "current DCS coordination cannot be trusted enough for authoritative decisions"

## HA decision boundary

`src/ha/decide.rs` currently uses a very hard gate:

- if `world.global.coordination.mode != DcsMode::Coordinated`, HA goes through `decide_degraded(world)`

So from the HA policy perspective, the meaningful split is:

- `Coordinated`: normal leader/follower decision logic is allowed
- `Degraded` or `NotTrusted`: the system takes fail-safe behavior

The current degraded-path outcomes include:

- a primary may be forced into `FailSafeGoal::PrimaryMustStop(...)`
- replicas may use `FailSafeGoal::ReplicaKeepFollowing(...)`
- nodes may wait for quorum with `FailSafeGoal::WaitForQuorum`
- public no-primary projections use `NoPrimaryProjection::DcsDegraded`

This is the critical docs point: trust mode is not cosmetic status text. It directly gates whether HA is allowed to do normal coordinated failover behavior.

## Example scenario requested by K2

The feature `tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature` is a good concrete narrative because it shows:

- a healthy cluster starting with one stable primary
- loss of a DCS quorum majority
- disappearance of the operator-visible primary
- every running node reporting `fail_safe`
- no dual-primary evidence during the transition
- restoration of DCS quorum
- recovery back to one stable primary

That feature is useful because it demonstrates the operator-facing consequence of the trust model:

- trust loss is intentionally conservative
- visibility of authority is withdrawn before the system is willing to claim a primary again

## One wording constraint for the final doc

Please avoid describing the current model as:

- "full quorum"
- "strict quorum calculation"
- "majority math"

Those phrases do not match the current code. The documentation should instead explain the actual implemented safety gate:

- etcd reachability
- self-member visibility
- minimal member-count rule
- HA fail-safe gating when the mode is not `Coordinated`
