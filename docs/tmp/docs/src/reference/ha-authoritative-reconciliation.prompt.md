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

docs/src/reference/ha-authoritative-reconciliation.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/explanation/ha-decision-engine.md
docs/src/explanation/introduction.md
docs/src/explanation/overview.md
docs/src/how-to/add-cluster-node.md
docs/src/how-to/bootstrap-cluster.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/configure-tls-security.md
docs/src/how-to/configure-tls.md
docs/src/how-to/debug-cluster-issues.md
docs/src/how-to/handle-network-partition.md
docs/src/how-to/handle-primary-failure.md
docs/src/how-to/monitor-via-metrics.md
docs/src/how-to/overview.md
docs/src/how-to/perform-switchover.md
docs/src/how-to/remove-cluster-node.md
docs/src/how-to/run-tests.md
docs/src/overview.md
docs/src/reference/dcs-state-model.md
docs/src/reference/debug-api.md
docs/src/reference/ha-authoritative-reconciliation.md
docs/src/reference/ha-decisions.md
docs/src/reference/http-api.md
docs/src/reference/overview.md
docs/src/reference/pgtm-cli.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/debug-api-usage.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md
docs/src/tutorial/overview.md
docs/src/tutorial/single-node-setup.md


# current docs summary context

===== docs/src/SUMMARY.md =====
# Summary

- [Overview](overview.md)

# Tutorials
- [Tutorials](tutorial/overview.md)
    - [First HA Cluster](tutorial/first-ha-cluster.md)
    - [Single-Node Setup](tutorial/single-node-setup.md)
    - [Observing a Failover Event](tutorial/observing-failover.md)
    - [Debug API Usage](tutorial/debug-api-usage.md)

# How-To

- [How-To](how-to/overview.md)
    - [Bootstrap a New Cluster from Zero State](how-to/bootstrap-cluster.md)
    - [Check Cluster Health](how-to/check-cluster-health.md)
    - [Add a Cluster Node](how-to/add-cluster-node.md)
    - [Configure TLS](how-to/configure-tls.md)
    - [Configure TLS Security](how-to/configure-tls-security.md)
    - [Debug Cluster Issues](how-to/debug-cluster-issues.md)
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

# Reference

- [Reference](reference/overview.md)
    - [HTTP API](reference/http-api.md)
    - [HA Decisions](reference/ha-decisions.md)
    - [Debug API](reference/debug-api.md)
    - [DCS State Model](reference/dcs-state-model.md)
    - [HA Authority Reconciliation Note](reference/ha-authoritative-reconciliation.md)
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

[features]
default = []

[dependencies]
clap = { version = "4.5.47", features = ["derive", "env"] }
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
sha2 = "0.10.9"
thiserror = "2.0.12"
tokio = { version = "1.44.1", features = ["sync", "rt", "rt-multi-thread", "macros", "time", "process", "net", "io-util", "fs", "signal"] }
tokio-postgres = "0.7.13"
toml = "0.8.20"
httparse = "1.10.1"
etcd-client = "0.14.1"
reqwest = { version = "0.12.24", default-features = false, features = ["blocking", "json", "rustls-tls"] }
rustls = { version = "0.23.28", features = ["ring"] }
rustls-pemfile = "2.2.0"
tokio-rustls = "0.26.4"
tracing = "0.1.41"
tracing-subscriber = "0.3.20"

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[dev-dependencies]
rcgen = "0.14.5"
x509-parser = "0.18.1"


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
src/api/fallback.rs
src/api/mod.rs
src/api/worker.rs
src/bin/pgtm.rs
src/bin/pgtuskmaster.rs
src/cli/args.rs
src/cli/client.rs
src/cli/config.rs
src/cli/connect.rs
src/cli/debug.rs
src/cli/error.rs
src/cli/mod.rs
src/cli/output.rs
src/cli/status.rs
src/config/defaults.rs
src/config/endpoint.rs
src/config/materialize.rs
src/config/mod.rs
src/config/parser.rs
src/config/schema.rs
src/dcs/etcd_store.rs
src/dcs/keys.rs
src/dcs/mod.rs
src/dcs/state.rs
src/dcs/store.rs
src/dcs/worker.rs
src/debug_api/mod.rs
src/debug_api/snapshot.rs
src/debug_api/view.rs
src/debug_api/worker.rs
src/ha/actions.rs
src/ha/apply.rs
src/ha/decide.rs
src/ha/decision.rs
src/ha/events.rs
src/ha/lower.rs
src/ha/mod.rs
src/ha/process_dispatch.rs
src/ha/source_conn.rs
src/ha/state.rs
src/ha/worker.rs
src/lib.rs
src/local_physical.rs
src/logging/event.rs
src/logging/mod.rs
src/logging/postgres_ingest.rs
src/logging/raw_record.rs
src/logging/tailer.rs
src/pginfo/conninfo.rs
src/pginfo/mod.rs
src/pginfo/query.rs
src/pginfo/state.rs
src/pginfo/worker.rs
src/postgres_managed.rs
src/postgres_managed_conf.rs
src/process/jobs.rs
src/process/mod.rs
src/process/state.rs
src/process/worker.rs
src/runtime/mod.rs
src/runtime/node.rs
src/state/errors.rs
src/state/ids.rs
src/state/mod.rs
src/state/time.rs
src/state/watch_state.rs
src/test_harness/auth.rs
src/test_harness/binaries.rs
src/test_harness/etcd3.rs
src/test_harness/ha_e2e/config.rs
src/test_harness/ha_e2e/handle.rs
src/test_harness/ha_e2e/mod.rs
src/test_harness/ha_e2e/ops.rs
src/test_harness/ha_e2e/startup.rs
src/test_harness/ha_e2e/util.rs
src/test_harness/mod.rs
src/test_harness/namespace.rs
src/test_harness/net_proxy.rs
src/test_harness/pg16.rs
src/test_harness/ports.rs
src/test_harness/provenance.rs
src/test_harness/runtime_config.rs
src/test_harness/signals.rs
src/test_harness/tls.rs
src/tls.rs
src/worker_contract_tests.rs
tests/bdd_api_http.rs
tests/bdd_state_watch.rs
tests/cli_binary.rs
tests/ha/support/multi_node.rs
tests/ha/support/observer.rs
tests/ha/support/partition.rs
tests/ha_multi_node_failover.rs
tests/ha_partition_isolation.rs
tests/nextest_config_contract.rs
tests/policy_e2e_api_only.rs


# docker and docs support file listing

docker/Dockerfile.dev
docker/Dockerfile.prod
docker/compose/docker-compose.cluster.yml
docker/compose/docker-compose.single.yml
docker/configs/cluster/config.toml
docker/configs/cluster/node-a/runtime.toml
docker/configs/cluster/node-b/runtime.toml
docker/configs/cluster/node-c/runtime.toml
docker/configs/common/pg_hba.conf
docker/configs/common/pg_ident.conf
docker/configs/single/node-a/runtime.toml
docker/entrypoint.sh
docker/secrets/postgres-superuser.password.example
docker/secrets/replicator.password.example
docker/secrets/rewinder.password.example
docs/book.toml
docs/draft/docs/src/explanation/architecture.md
docs/draft/docs/src/explanation/architecture.revised.md
docs/draft/docs/src/explanation/failure-modes.md
docs/draft/docs/src/explanation/failure-modes.revised.md
docs/draft/docs/src/explanation/ha-decision-engine.md
docs/draft/docs/src/explanation/introduction.md
docs/draft/docs/src/how-to/add-cluster-node.md
docs/draft/docs/src/how-to/bootstrap-cluster.md
docs/draft/docs/src/how-to/bootstrap-cluster.revised.md
docs/draft/docs/src/how-to/check-cluster-health.md
docs/draft/docs/src/how-to/configure-tls-security.md
docs/draft/docs/src/how-to/configure-tls.md
docs/draft/docs/src/how-to/debug-cluster-issues.md
docs/draft/docs/src/how-to/handle-network-partition.md
docs/draft/docs/src/how-to/handle-primary-failure.md
docs/draft/docs/src/how-to/handle-primary-failure.revised.md
docs/draft/docs/src/how-to/monitor-via-metrics.md
docs/draft/docs/src/how-to/perform-switchover.md
docs/draft/docs/src/how-to/perform-switchover.revised.md
docs/draft/docs/src/how-to/remove-cluster-node.md
docs/draft/docs/src/how-to/run-tests.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.revised.md
docs/draft/docs/src/reference/cli.revised.md
docs/draft/docs/src/reference/dcs-state-model.md
docs/draft/docs/src/reference/debug-api.md
docs/draft/docs/src/reference/ha-authoritative-reconciliation.md
docs/draft/docs/src/reference/ha-decisions.md
docs/draft/docs/src/reference/http-api.md
docs/draft/docs/src/reference/http-api.revised.md
docs/draft/docs/src/reference/pgtuskmaster-cli.md
docs/draft/docs/src/reference/pgtuskmaster-cli.revised.md
docs/draft/docs/src/reference/pgtuskmasterctl-cli.md
docs/draft/docs/src/reference/runtime-configuration.md
docs/draft/docs/src/reference/runtime-configuration.revised.md
docs/draft/docs/src/tutorial/debug-api-usage.md
docs/draft/docs/src/tutorial/first-ha-cluster.final.md
docs/draft/docs/src/tutorial/first-ha-cluster.md
docs/draft/docs/src/tutorial/first-ha-cluster.revised.md
docs/draft/docs/src/tutorial/observing-failover.md
docs/draft/docs/src/tutorial/observing-failover.revised.md
docs/draft/docs/src/tutorial/single-node-setup.md
docs/examples/docker-cluster-node-a.toml
docs/examples/docker-cluster-node-b.toml
docs/examples/docker-cluster-node-c.toml
docs/examples/docker-single-node-a.toml
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/explanation/ha-decision-engine.md
docs/src/explanation/introduction.md
docs/src/explanation/overview.md
docs/src/how-to/add-cluster-node.md
docs/src/how-to/bootstrap-cluster.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/configure-tls-security.md
docs/src/how-to/configure-tls.md
docs/src/how-to/debug-cluster-issues.md
docs/src/how-to/handle-network-partition.md
docs/src/how-to/handle-primary-failure.md
docs/src/how-to/monitor-via-metrics.md
docs/src/how-to/overview.md
docs/src/how-to/perform-switchover.md
docs/src/how-to/remove-cluster-node.md
docs/src/how-to/run-tests.md
docs/src/overview.md
docs/src/reference/dcs-state-model.md
docs/src/reference/debug-api.md
docs/src/reference/ha-authoritative-reconciliation.md
docs/src/reference/ha-decisions.md
docs/src/reference/http-api.md
docs/src/reference/overview.md
docs/src/reference/pgtm-cli.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/debug-api-usage.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md
docs/src/tutorial/overview.md
docs/src/tutorial/single-node-setup.md
docs/tmp/docs/src/reference/ha-authoritative-reconciliation.prompt.md


===== docs/src/reference/ha-authoritative-reconciliation.md =====
# HA Authority Reconciliation Note

This note freezes the target authority model for the HA startup/bootstrap/rejoin rewrite while code is in motion. It is intentionally narrower and more implementation-facing than the broader operator docs.

## DCS Facts

The redesign uses four authoritative DCS fact families under `/{scope}`:

- `/cluster/initialized`: durable record written only after successful first bootstrap
- `/cluster/identity`: durable record containing the authoritative cluster `system_identifier`, bootstrap winner, and bootstrap completion timestamp
- `/bootstrap`: leased bootstrap authority held only while one node is actively responsible for first-cluster bootstrap
- `/leader`: leased leader authority held only by the current elected leader

Member publication stays under `/member/{member_id}`. The member record remains the live observation surface and now also carries the pre-election descriptor used for offline ranking.

## Trust Vocabulary

`DcsTrust` is intentionally compressed to three authority states:

- `NotTrusted`: the store is unavailable or unhealthy
- `NoFreshQuorum`: the store is reachable but there is not enough fresh authority to elect or preserve a writable primary
- `FreshQuorum`: the store is healthy and fresh enough for bootstrap and leader-election decisions

Startup and steady-state HA obey the same rule: an initialized cluster may only run writable primary behavior while trust is `FreshQuorum`.

## Derived Per-Tick Plans

The runtime derives one cluster summary and one local desired state per tick from DCS facts plus local physical facts. These summaries are not persisted back into DCS.

```rust
enum ClusterMode {
    UninitializedNoBootstrapOwner,
    UninitializedBootstrapInProgress { holder: MemberId },
    InitializedLeaderPresent { leader: MemberId },
    InitializedNoLeaderFreshQuorum,
    InitializedNoLeaderNoFreshQuorum,
    DcsUnavailable,
}

enum DesiredNodeState {
    Bootstrap(BootstrapPlan),
    Primary(PrimaryPlan),
    Replica(ReplicaPlan),
    Quiescent(QuiescentReason),
    Fence(FencePlan),
}

enum BootstrapPlan {
    InitDb,
}

enum PrimaryPlan {
    KeepLeader,
    AcquireLeaderThenStartOrPromote,
}

enum ReplicaPlan {
    DirectFollow { leader: MemberId },
    RewindThenFollow { leader: MemberId },
    BasebackupThenFollow { leader: MemberId },
}

enum QuiescentReason {
    WaitingForBootstrapWinner,
    WaitingForAuthoritativeLeader,
    WaitingForFreshQuorum,
    UnsafeUninitializedPgData,
}

enum FencePlan {
    StopAndStayNonWritable,
}
```

## Deterministic Ordering

When `cluster_initialized = true` and no authoritative leader lease exists, the candidate set is ranked in this exact order:

1. matching expected cluster `system_identifier`
2. promotable/eligible state
3. higher `timeline_id`
4. higher `durable_end_lsn`
5. `postgres_runtime_class = running_healthy` over `offline_inspectable`
6. lowest lexical `member_id`

The ranking narrows which node may race for `/leader`, but the leased leader key remains the final serialization point.

When `cluster_initialized = false`, only nodes with `data_dir_kind = missing | empty` may race for `/bootstrap`. Nodes with unexpected non-empty `PGDATA` stay quiescent and surface a hard operator error.

## Local Physical Inspection Contract

Cold startup and steady-state HA share one authoritative local physical inspection path. It must publish at least:

- `data_dir_kind`
- `system_identifier`
- `pg_version`
- `timeline_id`
- `durable_end_lsn`
- control-file state
- whether the control data says the node was in recovery
- standby/recovery signal-file mode and conflicts
- eligibility for bootstrap, direct follow, rewind, and basebackup

`pg_controldata` output is the authoritative source for these physical facts. Signal-file inspection is supplemental and must not replace control-file facts.


===== src/dcs/state.rs =====
use std::{collections::BTreeMap, path::PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    state::{
        MemberId, StatePublisher, StateSubscriber, SystemIdentifier, TimelineId, UnixMillis,
        Version, WalLsn, WorkerStatus,
    },
};

use super::store::DcsStore;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DcsTrust {
    // Trust is derived per tick from store health plus fresh quorum visibility.
    FreshQuorum,
    NoFreshQuorum,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MemberRole {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MemberStateClass {
    EmptyOrMissingDataDir,
    InitializedInspectable,
    InvalidDataDir,
    ReplicaOnly,
    Promotable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PostgresRuntimeClass {
    RunningHealthy,
    OfflineInspectable,
    UnsafeLocalState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) member_id: MemberId,
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) api_url: Option<String>,
    pub(crate) role: MemberRole,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) write_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) durable_end_lsn: Option<WalLsn>,
    pub(crate) state_class: Option<MemberStateClass>,
    pub(crate) postgres_runtime_class: Option<PostgresRuntimeClass>,
    pub(crate) updated_at: UnixMillis,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderRecord {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequest {
    #[serde(default)]
    pub(crate) switchover_to: Option<MemberId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct BootstrapLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClusterInitializedRecord {
    pub(crate) initialized_by: MemberId,
    pub(crate) initialized_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClusterIdentityRecord {
    pub(crate) system_identifier: SystemIdentifier,
    pub(crate) bootstrapped_by: MemberId,
    pub(crate) bootstrapped_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCache {
    // These are the only durable or leased cluster facts cached from DCS.
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader: Option<LeaderRecord>,
    pub(crate) switchover: Option<SwitchoverRequest>,
    pub(crate) config: RuntimeConfig,
    pub(crate) cluster_initialized: Option<ClusterInitializedRecord>,
    pub(crate) cluster_identity: Option<ClusterIdentityRecord>,
    pub(crate) bootstrap_lock: Option<BootstrapLockRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsState {
    pub(crate) worker: WorkerStatus,
    pub(crate) trust: DcsTrust,
    pub(crate) cache: DcsCache,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
    pub(crate) poll_interval: Duration,
    pub(crate) local_postgres_host: String,
    pub(crate) local_postgres_port: u16,
    pub(crate) local_api_url: Option<String>,
    pub(crate) local_data_dir: PathBuf,
    pub(crate) local_postgres_binary: PathBuf,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsState>,
    pub(crate) store: Box<dyn DcsStore>,
    pub(crate) log: LogHandle,
    pub(crate) cache: DcsCache,
    pub(crate) last_published_pg_version: Option<Version>,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
}

pub(crate) struct BuildLocalMemberRecordInput<'a> {
    pub(crate) self_id: &'a MemberId,
    pub(crate) postgres_host: &'a str,
    pub(crate) postgres_port: u16,
    pub(crate) api_url: Option<&'a str>,
    pub(crate) pg_state: &'a PgInfoState,
    pub(crate) previous_record: Option<&'a MemberRecord>,
    pub(crate) now: UnixMillis,
    pub(crate) pg_version: Version,
}

pub(crate) fn evaluate_trust(
    etcd_healthy: bool,
    cache: &DcsCache,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    let Some(self_member) = cache.members.get(self_id) else {
        return DcsTrust::NoFreshQuorum;
    };
    if !member_record_is_fresh(self_member, cache, now) {
        return DcsTrust::NoFreshQuorum;
    }

    if !has_fresh_quorum(cache, now) {
        return DcsTrust::NoFreshQuorum;
    }

    DcsTrust::FreshQuorum
}

pub(crate) fn member_record_is_fresh(
    record: &MemberRecord,
    cache: &DcsCache,
    now: UnixMillis,
) -> bool {
    let max_age_ms = cache.config.ha.lease_ttl_ms;
    now.0.saturating_sub(record.updated_at.0) <= max_age_ms
}

fn fresh_member_count(cache: &DcsCache, now: UnixMillis) -> usize {
    cache
        .members
        .values()
        .filter(|record| member_record_is_fresh(record, cache, now))
        .count()
}

fn has_fresh_quorum(cache: &DcsCache, now: UnixMillis) -> bool {
    let fresh_members = fresh_member_count(cache, now);

    // The current runtime only knows the observed DCS member set. Until there is an explicit
    // configured membership source, multi-member quorum stays conservative: one fresh member is
    // only trusted in a single-member view, and any larger observed view requires at least two
    // fresh members.
    if cache.members.len() <= 1 {
        fresh_members == 1
    } else {
        fresh_members >= 2
    }
}

pub(crate) fn build_local_member_record(input: BuildLocalMemberRecordInput<'_>) -> MemberRecord {
    let BuildLocalMemberRecordInput {
        self_id,
        postgres_host,
        postgres_port,
        api_url,
        pg_state,
        previous_record,
        now,
        pg_version,
    } = input;
    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common
                .timeline
                .or(previous_record.and_then(|record| record.timeline)),
            write_lsn: previous_record.and_then(|record| record.write_lsn),
            replay_lsn: previous_record.and_then(|record| record.replay_lsn),
            system_identifier: previous_record.and_then(|record| record.system_identifier),
            durable_end_lsn: previous_record.and_then(|record| record.durable_end_lsn),
            state_class: previous_record.and_then(|record| record.state_class.clone()),
            postgres_runtime_class: previous_record
                .and_then(|record| record.postgres_runtime_class.clone()),
            updated_at: now,
            pg_version,
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: Some(*wal_lsn),
            replay_lsn: None,
            system_identifier: previous_record.and_then(|record| record.system_identifier),
            durable_end_lsn: previous_record
                .and_then(|record| record.durable_end_lsn)
                .or(Some(*wal_lsn)),
            state_class: previous_record.and_then(|record| record.state_class.clone()),
            postgres_runtime_class: Some(PostgresRuntimeClass::RunningHealthy),
            updated_at: now,
            pg_version,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: Some(*replay_lsn),
            system_identifier: previous_record.and_then(|record| record.system_identifier),
            durable_end_lsn: previous_record
                .and_then(|record| record.durable_end_lsn)
                .or(Some(*replay_lsn)),
            state_class: previous_record.and_then(|record| record.state_class.clone()),
            postgres_runtime_class: Some(PostgresRuntimeClass::RunningHealthy),
            updated_at: now,
            pg_version,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        pginfo::state::{PgConfig, PgInfoCommon, ReplicationSlotInfo},
        state::{Version, WorkerStatus},
    };

    use super::{
        build_local_member_record, evaluate_trust, BuildLocalMemberRecordInput, DcsCache,
        DcsTrust, LeaderRecord, MemberRecord, MemberRole,
    };
    use crate::{
        pginfo::state::{PgInfoState, Readiness, SqlStatus},
        state::{MemberId, TimelineId, UnixMillis, WalLsn},
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_cache() -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
        }
    }

    #[test]
    fn evaluate_trust_covers_all_outcomes() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();

        assert_eq!(
            evaluate_trust(false, &cache, &self_id, UnixMillis(1)),
            DcsTrust::NotTrusted
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::NoFreshQuorum
        );

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Unknown,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FreshQuorum
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::NoFreshQuorum
        );

        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FreshQuorum
        );
    }

    #[test]
    fn evaluate_trust_keeps_healthy_majority_when_leader_metadata_is_missing_or_stale() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();
        let fresh_time = UnixMillis(100);

        for member_id in ["node-a", "node-c"] {
            let member_id = MemberId(member_id.to_string());
            cache.members.insert(
                member_id.clone(),
                MemberRecord {
                    member_id,
                    postgres_host: "127.0.0.1".to_string(),
                    postgres_port: 5432,
                    api_url: None,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                    updated_at: fresh_time,
                    pg_version: Version(1),
                },
            );
        }
        cache.members.insert(
            MemberId("node-b".to_string()),
            MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "127.0.0.2".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });

        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(5_000)),
            DcsTrust::FreshQuorum
        );

        cache.members.remove(&MemberId("node-b".to_string()));
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(5_000)),
            DcsTrust::FreshQuorum
        );
    }

    #[test]
    fn evaluate_trust_stays_fail_safe_without_fresh_quorum() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Replica,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(100),
                pg_version: Version(1),
            },
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "127.0.0.2".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        cache.members.insert(
            MemberId("node-c".to_string()),
            MemberRecord {
                member_id: MemberId("node-c".to_string()),
                postgres_host: "127.0.0.3".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Replica,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });

        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::NoFreshQuorum
        );
    }

    fn common(sql: SqlStatus, readiness: Readiness) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
            readiness,
            timeline: Some(TimelineId(4)),
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(9)),
        }
    }

    #[test]
    fn build_local_member_record_maps_pg_variants() {
        let self_id = MemberId("node-a".to_string());
        let unknown = PgInfoState::Unknown {
            common: common(SqlStatus::Unknown, Readiness::Unknown),
        };
        let unknown_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.11",
            postgres_port: 5433,
            api_url: Some("http://node-a:8080"),
            pg_state: &unknown,
            previous_record: None,
            now: UnixMillis(10),
            pg_version: Version(11),
        });
        assert_eq!(unknown_record.postgres_host, "10.0.0.11".to_string());
        assert_eq!(unknown_record.postgres_port, 5433);
        assert_eq!(
            unknown_record.api_url.as_deref(),
            Some("http://node-a:8080")
        );
        assert_eq!(unknown_record.role, MemberRole::Unknown);
        assert_eq!(unknown_record.write_lsn, None);
        assert_eq!(unknown_record.replay_lsn, None);

        let primary = PgInfoState::Primary {
            common: common(SqlStatus::Healthy, Readiness::Ready),
            wal_lsn: WalLsn(101),
            slots: vec![ReplicationSlotInfo {
                name: "slot-a".to_string(),
            }],
        };
        let primary_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.12",
            postgres_port: 5434,
            api_url: Some("http://node-a:8081"),
            pg_state: &primary,
            previous_record: None,
            now: UnixMillis(12),
            pg_version: Version(13),
        });
        assert_eq!(primary_record.postgres_host, "10.0.0.12".to_string());
        assert_eq!(primary_record.postgres_port, 5434);
        assert_eq!(primary_record.role, MemberRole::Primary);
        assert_eq!(primary_record.write_lsn, Some(WalLsn(101)));
        assert_eq!(primary_record.replay_lsn, None);

        let replica = PgInfoState::Replica {
            common: common(SqlStatus::Healthy, Readiness::Ready),
            replay_lsn: WalLsn(22),
            follow_lsn: Some(WalLsn(23)),
            upstream: None,
        };
        let replica_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.13",
            postgres_port: 5435,
            api_url: None,
            pg_state: &replica,
            previous_record: None,
            now: UnixMillis(14),
            pg_version: Version(15),
        });
        assert_eq!(replica_record.postgres_host, "10.0.0.13".to_string());
        assert_eq!(replica_record.postgres_port, 5435);
        assert_eq!(replica_record.api_url, None);
        assert_eq!(replica_record.role, MemberRole::Replica);
        assert_eq!(replica_record.write_lsn, None);
        assert_eq!(replica_record.replay_lsn, Some(WalLsn(22)));
    }

    #[test]
    fn build_local_member_record_retains_last_known_wal_evidence_for_unknown_pg_state() {
        let self_id = MemberId("node-a".to_string());
        let previous = MemberRecord {
            member_id: self_id.clone(),
            postgres_host: "10.0.0.10".to_string(),
            postgres_port: 5432,
            api_url: None,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: Some(TimelineId(4)),
            write_lsn: Some(WalLsn(99)),
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
            updated_at: UnixMillis(8),
            pg_version: Version(9),
        };
        let unknown = PgInfoState::Unknown {
            common: common(SqlStatus::Unreachable, Readiness::NotReady),
        };

        let unknown_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.11",
            postgres_port: 5433,
            api_url: None,
            pg_state: &unknown,
            previous_record: Some(&previous),
            now: UnixMillis(10),
            pg_version: Version(11),
        });

        assert_eq!(unknown_record.role, MemberRole::Unknown);
        assert_eq!(unknown_record.timeline, Some(TimelineId(4)));
        assert_eq!(unknown_record.write_lsn, Some(WalLsn(99)));
        assert_eq!(unknown_record.replay_lsn, None);
    }
}


===== src/runtime/node.rs =====
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use thiserror::Error;
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    api::worker::ApiWorkerCtx,
    config::{load_runtime_config, validate_runtime_config, ConfigError, RuntimeConfig},
    dcs::{
        etcd_store::EtcdDcsStore,
        state::{ClusterIdentityRecord, ClusterInitializedRecord},
        state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx, MemberRole},
        store::{refresh_from_etcd_watch, DcsLeaderStore, DcsStore},
    },
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::source_conn::basebackup_source_from_member,
    ha::state::{
        HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
    },
    logging::{
        AppEvent, AppEventHeader, SeverityText, StructuredFields, SubprocessLineRecord,
        SubprocessStream,
    },
    local_physical::{inspect_local_physical_state, DataDirKind},
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, ProcessCommandRunner, ProcessExit, ReplicatorSourceConn,
            StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker::{build_command, system_now_unix_millis, timeout_for_kind, TokioCommandRunner},
    },
    state::{new_state_channel, MemberId, UnixMillis, WorkerStatus},
};

const STARTUP_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
const STARTUP_JOB_POLL_INTERVAL: Duration = Duration::from_millis(20);
const PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
enum StartupAction {
    AcquireBootstrapLockAndSeedConfig,
    PersistClusterBootstrapState,
    ReleaseBootstrapLock,
    ResetDataDir,
    RunJob(Box<ProcessJobKind>),
    StartPostgres(ManagedPostgresStartIntent),
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupMode {
    InitializePrimary {
        start_intent: ManagedPostgresStartIntent,
    },
    CloneReplica {
        leader_member_id: MemberId,
        source: ReplicatorSourceConn,
        start_intent: ManagedPostgresStartIntent,
        reset_data_dir: bool,
    },
    ResumeExisting {
        start_intent: ManagedPostgresStartIntent,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataDirState {
    Missing,
    Empty,
    Existing,
}

#[derive(Clone, Copy, Debug)]
enum RuntimeEventKind {
    StartupEntered,
    DataDirInspected,
    DcsCacheProbe,
    ModeSelected,
    ActionsPlanned,
    Action,
    Phase,
    SubprocessLogEmitFailed,
}

impl RuntimeEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StartupEntered => "runtime.startup.entered",
            Self::DataDirInspected => "runtime.startup.data_dir.inspected",
            Self::DcsCacheProbe => "runtime.startup.dcs_cache_probe",
            Self::ModeSelected => "runtime.startup.mode_selected",
            Self::ActionsPlanned => "runtime.startup.actions_planned",
            Self::Action => "runtime.startup.action",
            Self::Phase => "runtime.startup.phase",
            Self::SubprocessLogEmitFailed => "runtime.startup.subprocess_log_emit_failed",
        }
    }
}

fn runtime_event(
    kind: RuntimeEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "runtime", result),
    )
}

fn runtime_base_fields(cfg: &RuntimeConfig, startup_run_id: &str) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", cfg.dcs.scope.clone());
    fields.insert("member_id", cfg.cluster.member_id.clone());
    fields.insert("startup_run_id", startup_run_id.to_string());
    fields
}

fn startup_mode_label(startup_mode: &StartupMode) -> String {
    format!("{startup_mode:?}").to_lowercase()
}

fn startup_action_kind_label(action: &StartupAction) -> &'static str {
    match action {
        StartupAction::AcquireBootstrapLockAndSeedConfig => {
            "acquire_bootstrap_lock_and_seed_config"
        }
        StartupAction::PersistClusterBootstrapState => "persist_cluster_bootstrap_state",
        StartupAction::ReleaseBootstrapLock => "release_bootstrap_lock",
        StartupAction::ResetDataDir => "reset_data_dir",
        StartupAction::RunJob(_) => "run_job",
        StartupAction::StartPostgres(_) => "start_postgres",
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
    let log = logging.handle.clone();
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    let mut event = runtime_event(
        RuntimeEventKind::StartupEntered,
        "ok",
        SeverityText::Info,
        "runtime starting",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(&cfg, startup_run_id.as_str()).into_attributes());
    fields.insert(
        "logging.level",
        format!("{:?}", cfg.logging.level).to_lowercase(),
    );
    log.emit_app_event("runtime::run_node_from_config", event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
        })?;

    let process_defaults = process_defaults_from_config(&cfg);
    let startup_mode = plan_startup(&cfg, &process_defaults, &log, startup_run_id.as_str())?;
    execute_startup(
        &cfg,
        &process_defaults,
        &startup_mode,
        &log,
        startup_run_id.as_str(),
    )
    .await?;

    run_workers(cfg, process_defaults, log).await
}

fn process_defaults_from_config(cfg: &RuntimeConfig) -> ProcessDispatchDefaults {
    ProcessDispatchDefaults {
        postgres_binary: cfg.process.binaries.postgres.clone(),
        postgres_host: cfg.postgres.listen_host.clone(),
        postgres_port: cfg.postgres.listen_port,
        socket_dir: cfg.postgres.socket_dir.clone(),
        log_file: cfg.postgres.log_file.clone(),
        replicator_username: cfg.postgres.roles.replicator.username.clone(),
        replicator_auth: cfg.postgres.roles.replicator.auth.clone(),
        rewinder_username: cfg.postgres.roles.rewinder.username.clone(),
        rewinder_auth: cfg.postgres.roles.rewinder.auth.clone(),
        remote_dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
        remote_ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
        connect_timeout_s: cfg.postgres.connect_timeout_s,
        shutdown_mode: crate::process::jobs::ShutdownMode::Fast,
    }
}

fn advertised_postgres_port(cfg: &RuntimeConfig) -> u16 {
    cfg.postgres
        .advertise_port
        .unwrap_or(cfg.postgres.listen_port)
}

fn plan_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<StartupMode, RuntimeError> {
    plan_startup_with_probe(cfg, process_defaults, log, startup_run_id, probe_dcs_cache)
}

fn plan_startup_with_probe(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
    probe: impl Fn(&RuntimeConfig) -> Result<DcsCache, RuntimeError>,
) -> Result<StartupMode, RuntimeError> {
    let data_dir_state = match inspect_data_dir(
        &cfg.postgres.data_dir,
        &cfg.process.binaries.postgres,
    ) {
        Ok(value) => {
            let mut event = runtime_event(
                RuntimeEventKind::DataDirInspected,
                "ok",
                SeverityText::Debug,
                "data dir inspected",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert(
                "postgres.data_dir",
                cfg.postgres.data_dir.display().to_string(),
            );
            fields.insert("data_dir_state", format!("{value:?}").to_lowercase());
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|err| {
                    RuntimeError::StartupPlanning(format!(
                        "data dir inspection log emit failed: {err}"
                    ))
                })?;
            value
        }
        Err(err) => {
            let mut event = runtime_event(
                RuntimeEventKind::DataDirInspected,
                "failed",
                SeverityText::Error,
                "data dir inspection failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert(
                "postgres.data_dir",
                cfg.postgres.data_dir.display().to_string(),
            );
            fields.insert("error", err.to_string());
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|emit_err| {
                    RuntimeError::StartupPlanning(format!(
                        "data dir inspection log emit failed: {emit_err}"
                    ))
                })?;
            return Err(err);
        }
    };

    let cache = match probe(cfg) {
        Ok(cache) => {
            let mut event = runtime_event(
                RuntimeEventKind::DcsCacheProbe,
                "ok",
                SeverityText::Info,
                "startup dcs cache probe ok",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert("dcs_probe_status", "ok");
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|err| {
                    RuntimeError::StartupPlanning(format!("dcs cache probe log emit failed: {err}"))
                })?;
            Some(cache)
        }
        Err(err) => {
            let mut event = runtime_event(
                RuntimeEventKind::DcsCacheProbe,
                "failed",
                SeverityText::Warn,
                "startup dcs cache probe failed; continuing without cache",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert("error", err.to_string());
            fields.insert("dcs_probe_status", "failed");
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|emit_err| {
                    RuntimeError::StartupPlanning(format!(
                        "dcs cache probe log emit failed: {emit_err}"
                    ))
                })?;
            None
        }
    };

    let startup_mode = select_startup_mode(
        data_dir_state,
        cfg.postgres.data_dir.as_path(),
        cache.as_ref(),
        &cfg.cluster.member_id,
        process_defaults,
    )?;

    let mut event = runtime_event(
        RuntimeEventKind::ModeSelected,
        "ok",
        SeverityText::Info,
        "startup mode selected",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
    fields.insert("startup_mode", startup_mode_label(&startup_mode));
    log.emit_app_event("runtime::plan_startup", event)
        .map_err(|err| {
            RuntimeError::StartupPlanning(format!("startup mode log emit failed: {err}"))
        })?;

    Ok(startup_mode)
}

fn inspect_data_dir(data_dir: &Path, postgres_binary: &Path) -> Result<DataDirState, RuntimeError> {
    let inspected = inspect_local_physical_state(data_dir, postgres_binary)
        .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
    match inspected.data_dir_kind {
        DataDirKind::Missing => Ok(DataDirState::Missing),
        DataDirKind::Empty => Ok(DataDirState::Empty),
        DataDirKind::Initialized => Ok(DataDirState::Existing),
        DataDirKind::InvalidNonEmptyWithoutPgVersion => Err(RuntimeError::StartupPlanning(
            format!(
                "ambiguous data dir state: `{}` is non-empty but has no PG_VERSION",
                data_dir.display()
            ),
        )),
    }
}

fn probe_dcs_cache(cfg: &RuntimeConfig) -> Result<DcsCache, RuntimeError> {
    let mut store =
        EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope).map_err(|err| {
            RuntimeError::StartupPlanning(format!("failed to connect dcs for startup probe: {err}"))
        })?;

    let events = store.drain_watch_events().map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to read startup dcs events: {err}"))
    })?;

    let mut cache = DcsCache {
        members: BTreeMap::new(),
        leader: None,
        switchover: None,
        config: cfg.clone(),
        cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
    };

    refresh_from_etcd_watch(&cfg.dcs.scope, &mut cache, events).map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to decode startup dcs snapshot: {err}"))
    })?;

    Ok(cache)
}

fn select_startup_mode(
    data_dir_state: DataDirState,
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<StartupMode, RuntimeError> {
    let self_member_id = MemberId(self_member_id.to_string());
    match data_dir_state {
        DataDirState::Existing => {
            if let Some(leader_member) = leader_from_leader_key(cache, &self_member_id)
                .or_else(|| foreign_healthy_primary_member(cache, &self_member_id))
            {
                let source = basebackup_source_from_member(
                    &self_member_id,
                    &leader_member,
                    process_defaults,
                )
                .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
                return Ok(StartupMode::CloneReplica {
                    leader_member_id: leader_member.member_id.clone(),
                    start_intent: replica_start_intent_from_source(&source, data_dir),
                    source,
                    reset_data_dir: true,
                });
            }
            Ok(StartupMode::ResumeExisting {
                start_intent: select_resume_start_intent(
                    data_dir,
                    cache,
                    self_member_id.0.as_str(),
                    process_defaults,
                )?,
            })
        }
        DataDirState::Missing | DataDirState::Empty => {
            let cluster_initialized_present = cache
                .and_then(|snapshot| snapshot.cluster_initialized.as_ref())
                .is_some();
            let bootstrap_lock_present = cache
                .and_then(|snapshot| snapshot.bootstrap_lock.as_ref())
                .is_some();

            let leader = leader_from_leader_key(cache, &self_member_id).or_else(|| {
                if cluster_initialized_present {
                    foreign_healthy_primary_member(cache, &self_member_id)
                } else {
                    None
                }
            });

            match leader {
                Some(leader_member) => {
                    let source = basebackup_source_from_member(
                        &self_member_id,
                        &leader_member,
                        process_defaults,
                    )
                    .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
                    Ok(StartupMode::CloneReplica {
                        leader_member_id: leader_member.member_id.clone(),
                        start_intent: replica_start_intent_from_source(&source, data_dir),
                        source,
                        reset_data_dir: false,
                    })
                }
                None => {
                    if cluster_initialized_present {
                        Err(RuntimeError::StartupPlanning(
                            "cluster is already initialized but no healthy primary is available for basebackup"
                                .to_string(),
                        ))
                    } else if bootstrap_lock_present {
                        Err(RuntimeError::StartupPlanning(
                            "cluster bootstrap is already in progress but no authoritative leader is available yet"
                                .to_string(),
                        ))
                    } else {
                        Ok(StartupMode::InitializePrimary {
                            start_intent: ManagedPostgresStartIntent::primary(),
                        })
                    }
                }
            }
        }
    }
}

fn select_resume_start_intent(
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<ManagedPostgresStartIntent, RuntimeError> {
    let self_member_id = MemberId(self_member_id.to_string());
    let managed_recovery_state = crate::postgres_managed::inspect_managed_recovery_state(data_dir)
        .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
    let has_local_managed_replica_residue =
        managed_recovery_state != crate::postgres_managed_conf::ManagedRecoverySignal::None;

    let Some(cache) = cache else {
        if has_local_managed_replica_residue {
            return Err(RuntimeError::StartupPlanning(
                "existing postgres data dir contains managed replica recovery state but startup dcs cache probe was unavailable; cannot rebuild authoritative startup intent"
                    .to_string(),
            ));
        }
        return Ok(ManagedPostgresStartIntent::primary());
    };

    if cache
        .leader
        .as_ref()
        .map(|record| record.member_id == self_member_id)
        .unwrap_or(false)
    {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if let Some(leader_member) = leader_from_leader_key(Some(cache), &self_member_id)
        .or_else(|| foreign_healthy_primary_member(Some(cache), &self_member_id))
    {
        let source =
            basebackup_source_from_member(&self_member_id, &leader_member, process_defaults)
                .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
        return Ok(replica_start_intent_from_source(&source, data_dir));
    }

    if local_primary_member(cache, &self_member_id).is_some() {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if has_local_managed_replica_residue {
        return Err(RuntimeError::StartupPlanning(
            "existing postgres data dir contains managed replica recovery state but no healthy primary is available in DCS to rebuild authoritative managed config"
                .to_string(),
        ));
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn leader_from_leader_key(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    let snapshot = cache?;
    let leader_record = snapshot.leader.as_ref()?;
    if leader_record.member_id == *self_member_id {
        return None;
    }
    let member = snapshot.members.get(&leader_record.member_id)?;
    let eligible = member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy;
    if eligible {
        Some(member.clone())
    } else {
        None
    }
}

fn foreign_healthy_primary_member(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    cache?
        .members
        .values()
        .find(|member| {
            member.member_id != *self_member_id
                && member.role == MemberRole::Primary
                && member.sql == SqlStatus::Healthy
        })
        .cloned()
}

fn local_primary_member<'a>(
    cache: &'a DcsCache,
    self_member_id: &MemberId,
) -> Option<&'a crate::dcs::state::MemberRecord> {
    cache
        .members
        .get(self_member_id)
        .filter(|member| member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy)
}

fn replica_start_intent_from_source(
    source: &ReplicatorSourceConn,
    data_dir: &Path,
) -> ManagedPostgresStartIntent {
    ManagedPostgresStartIntent::replica(
        source.conninfo.clone(),
        managed_standby_auth_from_role_auth(&source.auth, data_dir),
        None,
    )
}

fn acquire_dcs_bootstrap_lock_and_seed_config(cfg: &RuntimeConfig) -> Result<(), String> {
    let config_path = format!("/{}/config", cfg.dcs.scope.trim_matches('/'));

    let mut store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope)
        .map_err(|err| format!("connect failed: {err}"))?;

    store
        .acquire_bootstrap_lease(&cfg.dcs.scope, &MemberId(cfg.cluster.member_id.clone()))
        .map_err(|err| format!("bootstrap lock acquire failed: {err}"))?;

    if let Some(init_cfg) = cfg.dcs.init.as_ref() {
        if init_cfg.write_on_bootstrap {
            let _seeded = store
                .put_path_if_absent(config_path.as_str(), init_cfg.payload_json.clone())
                .map_err(|err| format!("seed config failed at `{config_path}`: {err}"))?;
        }
    }

    Ok(())
}

fn persist_cluster_bootstrap_state(cfg: &RuntimeConfig) -> Result<(), String> {
    let inspected = inspect_local_physical_state(
        cfg.postgres.data_dir.as_path(),
        &cfg.process.binaries.postgres,
    )
    .map_err(|err| format!("post-bootstrap local inspection failed: {err}"))?;
    let system_identifier = inspected
        .system_identifier
        .ok_or_else(|| "post-bootstrap local inspection did not produce a system identifier".to_string())?;
    let initialized_at = system_now_unix_millis()
        .map_err(|err| format!("failed to capture bootstrap completion time: {err}"))?;

    let mut store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope)
        .map_err(|err| format!("connect failed: {err}"))?;

    store
        .write_cluster_identity(
            &cfg.dcs.scope,
            &ClusterIdentityRecord {
                system_identifier,
                bootstrapped_by: MemberId(cfg.cluster.member_id.clone()),
                bootstrapped_at: initialized_at,
            },
        )
        .map_err(|err| format!("cluster identity write failed: {err}"))?;
    store
        .write_cluster_initialized(
            &cfg.dcs.scope,
            &ClusterInitializedRecord {
                initialized_by: MemberId(cfg.cluster.member_id.clone()),
                initialized_at,
            },
        )
        .map_err(|err| format!("cluster initialized write failed: {err}"))?;
    Ok(())
}

fn release_dcs_bootstrap_lock(cfg: &RuntimeConfig) -> Result<(), String> {
    let mut store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope)
        .map_err(|err| format!("connect failed: {err}"))?;
    store
        .release_bootstrap_lease(&cfg.dcs.scope, &MemberId(cfg.cluster.member_id.clone()))
        .map_err(|err| format!("bootstrap lock release failed: {err}"))
}

async fn execute_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    startup_mode: &StartupMode,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<(), RuntimeError> {
    ensure_start_paths(process_defaults, &cfg.postgres.data_dir)?;

    let actions = build_startup_actions(cfg, startup_mode)?;

    let mut planned_event = runtime_event(
        RuntimeEventKind::ActionsPlanned,
        "ok",
        SeverityText::Debug,
        "startup actions planned",
    );
    let fields = planned_event.fields_mut();
    fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
    fields.insert("startup_mode", startup_mode_label(startup_mode));
    fields.insert("startup_actions_total", actions.len());
    log.emit_app_event("runtime::execute_startup", planned_event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("startup actions log emit failed: {err}"))
        })?;

    for (action_index, action) in actions.into_iter().enumerate() {
        let action_kind = startup_action_kind_label(&action);
        let mut action_fields = runtime_base_fields(cfg, startup_run_id);
        action_fields.insert("startup_mode", startup_mode_label(startup_mode));
        action_fields.insert("startup_action_index", action_index);
        action_fields.insert("startup_action_kind", action_kind);
        let mut started_event = runtime_event(
            RuntimeEventKind::Action,
            "started",
            SeverityText::Info,
            "startup action started",
        );
        started_event
            .fields_mut()
            .append_json_map(action_fields.clone().into_attributes());
        log.emit_app_event("runtime::execute_startup", started_event)
            .map_err(|err| {
                RuntimeError::StartupExecution(format!("startup action log emit failed: {err}"))
            })?;

        if let StartupAction::StartPostgres(_) = &action {
            emit_startup_phase(log, "start", "start postgres with managed config").map_err(
                |err| {
                    RuntimeError::StartupExecution(format!("startup phase log emit failed: {err}"))
                },
            )?;
        }

        let result = match action {
            StartupAction::AcquireBootstrapLockAndSeedConfig => {
                acquire_dcs_bootstrap_lock_and_seed_config(cfg).map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "dcs bootstrap lock claim failed: {err}"
                    ))
                })?;
                Ok(())
            }
            StartupAction::PersistClusterBootstrapState => persist_cluster_bootstrap_state(cfg)
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "persist cluster bootstrap state failed: {err}"
                    ))
                }),
            StartupAction::ReleaseBootstrapLock => release_dcs_bootstrap_lock(cfg).map_err(
                |err| RuntimeError::StartupExecution(format!("release bootstrap lock failed: {err}")),
            ),
            StartupAction::ResetDataDir => reset_data_dir(cfg.postgres.data_dir.as_path()),
            StartupAction::RunJob(job) => run_startup_job(cfg, *job, log).await,
            StartupAction::StartPostgres(start_intent) => {
                run_start_job(cfg, process_defaults, &start_intent, log).await
            }
        };

        match result {
            Ok(()) => {
                let mut done_event = runtime_event(
                    RuntimeEventKind::Action,
                    "ok",
                    SeverityText::Info,
                    "startup action completed",
                );
                done_event
                    .fields_mut()
                    .append_json_map(action_fields.into_attributes());
                log.emit_app_event("runtime::execute_startup", done_event)
                    .map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup action log emit failed: {err}"
                        ))
                    })?;
            }
            Err(err) => {
                let mut failed_event = runtime_event(
                    RuntimeEventKind::Action,
                    "failed",
                    SeverityText::Error,
                    "startup action failed",
                );
                let fields = failed_event.fields_mut();
                fields.append_json_map(action_fields.into_attributes());
                fields.insert("error", err.to_string());
                log.emit_app_event("runtime::execute_startup", failed_event)
                    .map_err(|emit_err| {
                        RuntimeError::StartupExecution(format!(
                            "startup action failure log emit failed: {emit_err}"
                        ))
                    })?;
                return Err(err);
            }
        };
    }

    Ok(())
}

fn emit_startup_phase(
    log: &crate::logging::LogHandle,
    phase: &str,
    detail: &str,
) -> Result<(), crate::logging::LogError> {
    let mut event = runtime_event(
        RuntimeEventKind::Phase,
        "ok",
        SeverityText::Info,
        format!("startup phase={phase} ({detail})"),
    );
    let fields = event.fields_mut();
    fields.insert("startup.phase", phase.to_string());
    fields.insert("startup.detail", detail.to_string());
    log.emit_app_event("startup", event)
}

fn build_startup_actions(
    cfg: &RuntimeConfig,
    startup_mode: &StartupMode,
) -> Result<Vec<StartupAction>, RuntimeError> {
    match startup_mode {
        StartupMode::InitializePrimary { start_intent } => Ok(vec![
            StartupAction::AcquireBootstrapLockAndSeedConfig,
            StartupAction::RunJob(Box::new(ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser_username: cfg.postgres.roles.superuser.username.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::PersistClusterBootstrapState,
            StartupAction::ReleaseBootstrapLock,
            StartupAction::StartPostgres(start_intent.clone()),
        ]),
        StartupMode::CloneReplica {
            source,
            start_intent,
            reset_data_dir,
            ..
        } => {
            let mut actions = Vec::new();
            if *reset_data_dir {
                actions.push(StartupAction::ResetDataDir);
            }
            actions.push(StartupAction::RunJob(Box::new(ProcessJobKind::BaseBackup(
                BaseBackupSpec {
                    data_dir: cfg.postgres.data_dir.clone(),
                    source: source.clone(),
                    timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
                },
            ))));
            actions.push(StartupAction::StartPostgres(start_intent.clone()));
            Ok(actions)
        }
        StartupMode::ResumeExisting { start_intent } => {
            if has_postmaster_pid(&cfg.postgres.data_dir) {
                Ok(Vec::new())
            } else {
                Ok(vec![StartupAction::StartPostgres(start_intent.clone())])
            }
        }
    }
}

fn ensure_start_paths(
    process_defaults: &ProcessDispatchDefaults,
    data_dir: &Path,
) -> Result<(), RuntimeError> {
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres data dir parent `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    fs::create_dir_all(data_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres data dir `{}`: {err}",
            data_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to set postgres data dir permissions on `{}`: {err}",
                data_dir.display()
            ))
        })?;
    }

    fs::create_dir_all(&process_defaults.socket_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres socket dir `{}`: {err}",
            process_defaults.socket_dir.display()
        ))
    })?;

    if let Some(log_parent) = process_defaults.log_file.parent() {
        fs::create_dir_all(log_parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres log dir `{}`: {err}",
                log_parent.display()
            ))
        })?;
    }

    Ok(())
}

fn has_postmaster_pid(data_dir: &Path) -> bool {
    data_dir.join("postmaster.pid").exists()
}

fn reset_data_dir(data_dir: &Path) -> Result<(), RuntimeError> {
    if data_dir.exists() {
        fs::remove_dir_all(data_dir).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to remove postgres data dir `{}` before basebackup: {err}",
                data_dir.display()
            ))
        })?;
    }
    fs::create_dir_all(data_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to recreate postgres data dir `{}` before basebackup: {err}",
            data_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to set postgres data dir permissions on `{}` before basebackup: {err}",
                data_dir.display()
            ))
        })?;
    }

    Ok(())
}

async fn run_start_job(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    start_intent: &ManagedPostgresStartIntent,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let managed = crate::postgres_managed::materialize_managed_postgres_config(cfg, start_intent)
        .map_err(|err| {
        RuntimeError::StartupExecution(format!("materialize managed postgres config failed: {err}"))
    })?;
    run_startup_job(
        cfg,
        ProcessJobKind::StartPostgres(StartPostgresSpec {
            data_dir: cfg.postgres.data_dir.clone(),
            config_file: managed.postgresql_conf_path,
            log_file: process_defaults.log_file.clone(),
            wait_seconds: Some(30),
            timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
        }),
        log,
    )
    .await
}

async fn run_startup_job(
    cfg: &RuntimeConfig,
    job: ProcessJobKind,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let mut runner = TokioCommandRunner;
    let timeout_ms = timeout_for_kind(&job, &cfg.process);
    let job_id = crate::state::JobId(format!("startup-{}", std::process::id()));
    let command = build_command(
        &cfg.process,
        &job_id,
        &job,
        cfg.logging.capture_subprocess_output,
    )
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("startup command build failed: {err}"))
    })?;
    let log_identity = command.log_identity.clone();
    let command_display = format!("{} {}", command.program.display(), command.args.join(" "));

    let mut handle = runner.spawn(command).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "startup command spawn failed for `{command_display}`: {err}"
        ))
    })?;

    let started = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
    let deadline = started.0.saturating_add(timeout_ms);

    loop {
        let lines = handle
            .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
            .await
            .map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup process output drain failed: {err}"
                ))
            })?;
        for line in lines {
            if let Err(err) = emit_startup_subprocess_line(log, &log_identity, line.clone()) {
                let mut event = runtime_event(
                    RuntimeEventKind::SubprocessLogEmitFailed,
                    "failed",
                    SeverityText::Warn,
                    "startup subprocess line emit failed",
                );
                let fields = event.fields_mut();
                fields.insert("job_id", log_identity.job_id.0.clone());
                fields.insert("job_kind", log_identity.job_kind.clone());
                fields.insert("binary", log_identity.binary.clone());
                fields.insert(
                    "stream",
                    match line.stream {
                        crate::process::jobs::ProcessOutputStream::Stdout => "stdout",
                        crate::process::jobs::ProcessOutputStream::Stderr => "stderr",
                    },
                );
                fields.insert("bytes_len", line.bytes.len());
                fields.insert("error", err.to_string());
                log.emit_app_event("runtime::run_startup_job", event)
                    .map_err(|emit_err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess emit failure log emit failed: {emit_err}"
                        ))
                    })?;
            }
        }

        match handle.poll_exit().map_err(|err| {
            RuntimeError::StartupExecution(format!("startup process poll failed: {err}"))
        })? {
            Some(ProcessExit::Success) => return Ok(()),
            Some(ProcessExit::Failure { code }) => {
                let lines = handle
                    .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
                    .await
                    .map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup process output drain failed: {err}"
                        ))
                    })?;
                for line in lines {
                    emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess line emit failed: {err}"
                        ))
                    })?;
                }
                return Err(RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` exited unsuccessfully (code: {code:?})"
                )));
            }
            None => {}
        }

        let now = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
        if now.0 >= deadline {
            handle.cancel().await.map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` timeout cancellation failed: {err}"
                ))
            })?;
            let lines = handle
                .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
                .await
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup process output drain failed: {err}"
                    ))
                })?;
            for line in lines {
                emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup subprocess line emit failed: {err}"
                    ))
                })?;
            }
            return Err(RuntimeError::StartupExecution(format!(
                "startup command `{command_display}` timed out after {timeout_ms} ms"
            )));
        }

        tokio::time::sleep(STARTUP_JOB_POLL_INTERVAL).await;
    }
}

fn emit_startup_subprocess_line(
    log: &crate::logging::LogHandle,
    identity: &crate::process::jobs::ProcessLogIdentity,
    line: crate::process::jobs::ProcessOutputLine,
) -> Result<(), crate::logging::LogError> {
    let stream = match line.stream {
        crate::process::jobs::ProcessOutputStream::Stdout => SubprocessStream::Stdout,
        crate::process::jobs::ProcessOutputStream::Stderr => SubprocessStream::Stderr,
    };

    log.emit_raw_record(
        SubprocessLineRecord::new(
            crate::logging::LogProducer::PgTool,
            "startup",
            identity.job_id.0.clone(),
            identity.job_kind.clone(),
            identity.binary.clone(),
            stream,
            line.bytes,
        )
        .into_raw_record()?,
    )
}

async fn run_workers(
    cfg: RuntimeConfig,
    process_defaults: ProcessDispatchDefaults,
    log: crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let now = now_unix_millis()?;

    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), now);
    let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg_state(), now);

    let initial_dcs = DcsState {
        worker: WorkerStatus::Starting,
        trust: DcsTrust::NotTrusted,
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
        },
        last_refresh_at: None,
    };
    let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, now);

    let initial_process = ProcessState::Idle {
        worker: WorkerStatus::Starting,
        last_outcome: None,
    };
    let (process_publisher, process_subscriber) = new_state_channel(initial_process.clone(), now);

    let initial_ha = HaState {
        worker: WorkerStatus::Starting,
        phase: HaPhase::Init,
        tick: 0,
        decision: crate::ha::decision::HaDecision::NoChange,
    };
    let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha, now);

    let initial_debug_snapshot = build_snapshot(
        &DebugSnapshotCtx {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
        },
        now,
        0,
        &[],
        &[],
    );
    let (debug_publisher, debug_subscriber) = new_state_channel(initial_debug_snapshot, now);

    let self_id = MemberId(cfg.cluster.member_id.clone());
    let scope = cfg.dcs.scope.clone();

    let pg_ctx = crate::pginfo::state::PgInfoWorkerCtx {
        self_id: self_id.clone(),
        postgres_conninfo: local_postgres_conninfo(
            &process_defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        ),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        publisher: pg_publisher,
        log: log.clone(),
        last_emitted_sql_status: None,
    };

    let dcs_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;
    let dcs_ctx = DcsWorkerCtx {
        self_id: self_id.clone(),
        scope: scope.clone(),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        local_postgres_host: cfg.postgres.listen_host.clone(),
        local_postgres_port: advertised_postgres_port(&cfg),
        local_api_url: advertised_operator_api_url(&cfg),
        local_data_dir: cfg.postgres.data_dir.clone(),
        local_postgres_binary: cfg.process.binaries.postgres.clone(),
        pg_subscriber: pg_subscriber.clone(),
        publisher: dcs_publisher,
        store: Box::new(dcs_store),
        log: log.clone(),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
        },
        last_published_pg_version: None,
        last_emitted_store_healthy: None,
        last_emitted_trust: None,
    };

    let (process_inbox_tx, process_inbox_rx) = mpsc::unbounded_channel();
    let process_ctx = ProcessWorkerCtx {
        poll_interval: PROCESS_WORKER_POLL_INTERVAL,
        config: cfg.process.clone(),
        log: log.clone(),
        capture_subprocess_output: cfg.logging.capture_subprocess_output,
        state: initial_process,
        publisher: process_publisher,
        inbox: process_inbox_rx,
        inbox_disconnected_logged: false,
        command_runner: Box::new(TokioCommandRunner),
        active_runtime: None,
        last_rejection: None,
        now: Box::new(system_now_unix_millis),
    };

    let ha_store = EtcdDcsStore::connect_with_leader_lease(
        cfg.dcs.endpoints.clone(),
        &scope,
        cfg.ha.lease_ttl_ms,
    )
    .map_err(|err| RuntimeError::Worker(format!("ha store connect failed: {err}")))?;
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        process_inbox: process_inbox_tx,
        dcs_store: Box::new(ha_store),
        scope: scope.clone(),
        self_id: self_id.clone(),
    });
    ha_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    ha_ctx.now = Box::new(system_now_unix_millis);
    ha_ctx.process_defaults = process_defaults;
    ha_ctx.log = log.clone();

    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        ha_subscriber: ha_subscriber.clone(),
    });
    debug_ctx.app = AppLifecycle::Running;
    debug_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    debug_ctx.now = Box::new(system_now_unix_millis);

    let api_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("api store connect failed: {err}")))?;
    let listener = TcpListener::bind(cfg.api.listen_addr)
        .await
        .map_err(|err| RuntimeError::ApiBind {
            listen_addr: cfg.api.listen_addr,
            message: err.to_string(),
        })?;
    let mut api_ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(api_store), log.clone());
    api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
    let server_tls = crate::tls::build_rustls_server_config(&cfg.api.security.tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls config build failed: {err}")))?;
    api_ctx
        .configure_tls(cfg.api.security.tls.mode, server_tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls configure failed: {err}")))?;
    let require_client_cert = match cfg.api.security.tls.client_auth.as_ref() {
        Some(auth) => auth.require_client_cert,
        None => false,
    };
    api_ctx.set_require_client_cert(require_client_cert);

    tokio::try_join!(
        crate::pginfo::worker::run(pg_ctx),
        crate::dcs::worker::run(dcs_ctx),
        crate::process::worker::run(process_ctx),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        crate::ha::worker::run(ha_ctx),
        crate::debug_api::worker::run(debug_ctx),
        crate::api::worker::run(api_ctx),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}

fn advertised_operator_api_url(cfg: &RuntimeConfig) -> Option<String> {
    if let Some(api_url) = cfg.pgtm.as_ref().and_then(|pgtm| pgtm.api_url.clone()) {
        return Some(api_url);
    }

    if cfg.api.listen_addr.ip().is_unspecified() {
        return None;
    }

    let scheme = match cfg.api.security.tls.mode {
        crate::config::ApiTlsMode::Disabled => "http",
        crate::config::ApiTlsMode::Optional | crate::config::ApiTlsMode::Required => "https",
    };
    Some(format!("{scheme}://{}", cfg.api.listen_addr))
}

fn local_postgres_conninfo(
    process_defaults: &ProcessDispatchDefaults,
    identity: &crate::config::PostgresConnIdentityConfig,
    superuser_username: &str,
    connect_timeout_s: u32,
) -> crate::pginfo::state::PgConnInfo {
    crate::pginfo::state::PgConnInfo {
        host: process_defaults.socket_dir.display().to_string(),
        port: process_defaults.postgres_port,
        user: superuser_username.to_string(),
        dbname: identity.dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(connect_timeout_s),
        ssl_mode: identity.ssl_mode,
        options: None,
    }
}

fn initial_pg_state() -> PgInfoState {
    PgInfoState::Unknown {
        common: PgInfoCommon {
            worker: WorkerStatus::Starting,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: None,
        },
    }
}

fn now_unix_millis() -> Result<UnixMillis, RuntimeError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RuntimeError::Time(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| RuntimeError::Time(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs, io,
        path::{Path, PathBuf},
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::pginfo::conninfo::PgSslMode;
    use crate::{
        config::{PostgresConfig, RuntimeConfig},
        dcs::state::{DcsCache, LeaderRecord, MemberRecord, MemberRole},
        logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        advertised_postgres_port, build_startup_actions, inspect_data_dir, plan_startup_with_probe,
        process_defaults_from_config, replica_start_intent_from_source,
        select_resume_start_intent, select_startup_mode, DataDirState, StartupAction,
        StartupMode,
    };
    use crate::postgres_managed_conf::{
        managed_standby_auth_from_role_auth, ManagedPostgresStartIntent,
    };
    use crate::process::state::ProcessJobKind;

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(PathBuf::from("/tmp/pgtuskmaster-test-data"))
            .build()
    }

    fn temp_path(label: &str) -> PathBuf {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-runtime-{label}-{millis}-{}",
            std::process::id()
        ))
    }

    fn remove_if_exists(path: &PathBuf) -> Result<(), io::Error> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    fn initdb_data_dir(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let output = std::process::Command::new(&cfg.process.binaries.initdb)
            .arg("-D")
            .arg(path)
            .arg("-U")
            .arg("postgres")
            .arg("--auth=trust")
            .arg("--no-sync")
            .env("LC_ALL", "C")
            .output()?;
        if !output.status.success() {
            return Err(format!(
                "initdb failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    #[test]
    fn inspect_data_dir_classifies_missing_empty_and_existing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let missing = temp_path("missing");
        remove_if_exists(&missing)?;
        assert_eq!(
            inspect_data_dir(&missing, Path::new("/usr/bin/postgres"))?,
            DataDirState::Missing
        );

        let empty = temp_path("empty");
        remove_if_exists(&empty)?;
        fs::create_dir_all(&empty)?;
        assert_eq!(
            inspect_data_dir(&empty, Path::new("/usr/bin/postgres"))?,
            DataDirState::Empty
        );

        let existing = temp_path("existing");
        remove_if_exists(&existing)?;
        initdb_data_dir(&existing)?;
        assert_eq!(
            inspect_data_dir(&existing, Path::new("/usr/bin/postgres"))?,
            DataDirState::Existing
        );

        remove_if_exists(&empty)?;
        remove_if_exists(&existing)?;
        Ok(())
    }

    #[test]
    fn plan_startup_emits_data_dir_and_mode_events_without_network_probe(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        let dir = temp_path("plan-startup-log");
        remove_if_exists(&dir)?;
        cfg.postgres.data_dir = dir.clone();

        let process_defaults = process_defaults_from_config(&cfg);
        let (log, sink) = test_log_handle();

        let _startup_mode =
            plan_startup_with_probe(&cfg, &process_defaults, &log, "run-1", |_cfg| {
                Ok(DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: cfg.clone(),
                    cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
                })
            })?;

        let inspected = sink.collect_matching(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "runtime.startup.data_dir.inspected")
                .unwrap_or(false)
        })?;
        if inspected.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.data_dir.inspected event",
            )));
        }

        let probe = sink.collect_matching(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "runtime.startup.dcs_cache_probe")
                .unwrap_or(false)
        })?;
        if probe.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.dcs_cache_probe event",
            )));
        }

        let mode_selected = sink.collect_matching(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "runtime.startup.mode_selected")
                .unwrap_or(false)
        })?;
        if mode_selected.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.mode_selected event",
            )));
        }

        remove_if_exists(&dir)?;
        Ok(())
    }

    #[test]
    fn inspect_data_dir_rejects_ambiguous_partial_state() -> Result<(), Box<dyn std::error::Error>>
    {
        let ambiguous = temp_path("ambiguous");
        remove_if_exists(&ambiguous)?;
        fs::create_dir_all(&ambiguous)?;
        fs::write(ambiguous.join("postgresql.conf"), b"# test\n")?;

        let result = inspect_data_dir(&ambiguous, Path::new("/usr/bin/postgres"));
        assert!(result.is_err());

        remove_if_exists(&ambiguous)?;
        Ok(())
    }

    #[test]
    fn select_startup_mode_prefers_clone_when_foreign_healthy_leader_exists(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: Some(LeaderRecord {
                member_id: leader_id.clone(),
            }),
            switchover: None,
            config: cfg.clone(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
        };

        let data_dir = temp_path("startup-mode-clone");
        remove_if_exists(&data_dir)?;
        let mode = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        if let StartupMode::CloneReplica {
            leader_member_id,
            source,
            reset_data_dir,
            ..
        } = mode
        {
            assert_eq!(leader_member_id, leader_id);
            assert!(!reset_data_dir);
            assert_eq!(
                source,
                crate::ha::source_conn::basebackup_source_from_member(
                    &MemberId("node-a".to_string()),
                    cache.members.get(&leader_id).ok_or_else(|| {
                        io::Error::other("leader member missing from startup test cache")
                    })?,
                    &defaults,
                )?
            );
        }
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_initialize_when_no_leader_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let data_dir = temp_path("startup-mode-init");
        remove_if_exists(&data_dir)?;

        let mode = select_startup_mode(DataDirState::Empty, &data_dir, None, "node-a", &defaults)?;

        assert_eq!(
            mode,
            StartupMode::InitializePrimary {
                start_intent: ManagedPostgresStartIntent::primary(),
            }
        );
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_resume_when_pgdata_exists() -> Result<(), Box<dyn std::error::Error>>
    {
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let data_dir = temp_path("startup-mode-resume");
        remove_if_exists(&data_dir)?;
        let mode =
            select_startup_mode(DataDirState::Existing, &data_dir, None, "node-a", &defaults)?;
        assert_eq!(
            mode,
            StartupMode::ResumeExisting {
                start_intent: ManagedPostgresStartIntent::primary(),
            }
        );
        Ok(())
    }

    #[test]
    fn select_startup_mode_rebuilds_existing_replica_from_foreign_healthy_leader(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: Some(LeaderRecord {
                member_id: leader_id.clone(),
            }),
            switchover: None,
            config: cfg.clone(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
        };

        let data_dir = temp_path("startup-mode-existing-clone");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;
        fs::write(data_dir.join("PG_VERSION"), b"16\n")?;

        let mode = select_startup_mode(
            DataDirState::Existing,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        if let StartupMode::CloneReplica {
            leader_member_id,
            source,
            reset_data_dir,
            ..
        } = mode
        {
            assert_eq!(leader_member_id, leader_id);
            assert!(reset_data_dir);
            assert_eq!(
                source,
                crate::ha::source_conn::basebackup_source_from_member(
                    &MemberId("node-a".to_string()),
                    cache.members.get(&leader_id).ok_or_else(|| {
                        io::Error::other("leader member missing from startup test cache")
                    })?,
                    &defaults,
                )?
            );
        }

        remove_if_exists(&data_dir)?;
        Ok(())
    }

    #[test]
    fn actions_for_startup_reset_existing_data_dir_before_clone_basebackup(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let leader_id = MemberId("node-b".to_string());
        let leader = MemberRecord {
            member_id: leader_id.clone(),
            postgres_host: "10.0.0.20".to_string(),
            postgres_port: 5440,
            api_url: None,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
            updated_at: UnixMillis(1),
            pg_version: Version(1),
        };
        let source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            &leader,
            &defaults,
        )?;
        let mode = StartupMode::CloneReplica {
            leader_member_id: leader_id,
            source: source.clone(),
            start_intent: replica_start_intent_from_source(&source, cfg.postgres.data_dir.as_path()),
            reset_data_dir: true,
        };

        let actions = build_startup_actions(&cfg, &mode)?;
        assert!(matches!(actions.first(), Some(StartupAction::ResetDataDir)));
        assert!(matches!(
            actions.get(1),
            Some(StartupAction::RunJob(job))
                if matches!(job.as_ref(), ProcessJobKind::BaseBackup(_))
        ));
        assert!(matches!(
            actions.get(2),
            Some(StartupAction::StartPostgres(_))
        ));

        Ok(())
    }

    #[test]
    fn select_resume_start_intent_prefers_dcs_leader_over_local_auto_conf(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let data_dir = temp_path("resume-dcs-authority");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;

        let runtime_config = RuntimeConfig {
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                ..cfg.postgres.clone()
            },
            ..cfg.clone()
        };
        crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.30".to_string(),
                    port: 5439,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    &data_dir,
                ),
                Some("slot_local".to_string()),
            ),
        )?;
        fs::write(
            data_dir.join("postgresql.auto.conf"),
            "primary_conninfo = 'host=192.0.2.99 port=6543 user=bad dbname=postgres'\n",
        )?;

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        let cache = DcsCache {
            members,
            leader: Some(LeaderRecord {
                member_id: leader_id.clone(),
            }),
            switchover: None,
            config: runtime_config.clone(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
        };

        let intent = select_resume_start_intent(&data_dir, Some(&cache), "node-a", &defaults)?;
        let expected_source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            cache
                .members
                .get(&leader_id)
                .ok_or_else(|| io::Error::other("leader missing from test cache"))?,
            &defaults,
        )?;
        assert_eq!(
            intent,
            ManagedPostgresStartIntent::replica(
                expected_source.conninfo,
                managed_standby_auth_from_role_auth(&expected_source.auth, &data_dir,),
                None,
            )
        );

        remove_if_exists(&data_dir)?;
        Ok(())
    }

    #[test]
    fn select_resume_start_intent_rejects_local_replica_state_without_dcs_authority(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let data_dir = temp_path("resume-without-dcs");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;

        let runtime_config = RuntimeConfig {
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                ..cfg.postgres.clone()
            },
            ..cfg.clone()
        };
        crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.30".to_string(),
                    port: 5439,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    &data_dir,
                ),
                Some("slot_local".to_string()),
            ),
        )?;

        let result = select_resume_start_intent(&data_dir, None, "node-a", &defaults);
        assert!(matches!(
            result,
            Err(super::RuntimeError::StartupPlanning(_))
        ));

        remove_if_exists(&data_dir)?;
        Ok(())
    }

    #[test]
    fn select_startup_mode_rejects_initialize_when_init_lock_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            cluster_initialized: Some(crate::dcs::state::ClusterInitializedRecord {
                initialized_by: MemberId("node-other".to_string()),
                initialized_at: UnixMillis(1),
            }),
            cluster_identity: None,
            bootstrap_lock: None,
        };

        let data_dir = temp_path("startup-mode-init-lock");
        remove_if_exists(&data_dir)?;
        let result = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        );

        assert!(matches!(
            result,
            Err(super::RuntimeError::StartupPlanning(_))
        ));
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_member_records_when_init_lock_present_and_leader_missing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let primary_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            primary_id.clone(),
            MemberRecord {
                member_id: primary_id.clone(),
                postgres_host: "10.0.0.21".to_string(),
                postgres_port: 5441,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: None,
            switchover: None,
            config: cfg.clone(),
            cluster_initialized: Some(crate::dcs::state::ClusterInitializedRecord {
                initialized_by: MemberId("node-init".to_string()),
                initialized_at: UnixMillis(1),
            }),
            cluster_identity: None,
            bootstrap_lock: None,
        };

        let data_dir = temp_path("startup-mode-member-fallback");
        remove_if_exists(&data_dir)?;
        let mode = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        Ok(())
    }

    #[test]
    fn runtime_uses_role_specific_users_for_dsn_clone_and_rewind(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        cfg.postgres.roles.superuser.username = "su_admin".to_string();
        cfg.postgres.roles.replicator.username = "repl_user".to_string();
        cfg.postgres.roles.rewinder.username = "rewind_user".to_string();
        cfg.postgres.local_conn_identity.user = "su_admin".to_string();
        cfg.postgres.rewind_conn_identity.user = "rewind_user".to_string();

        let defaults = super::process_defaults_from_config(&cfg);
        assert_eq!(defaults.replicator_username, "repl_user");
        assert_eq!(defaults.rewinder_username, "rewind_user");

        let local_conninfo = super::local_postgres_conninfo(
            &defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        );
        assert_eq!(local_conninfo.user, "su_admin");

        let leader_source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            &MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "10.0.0.30".to_string(),
                postgres_port: 5442,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
            &defaults,
        )?;
        assert_eq!(leader_source.conninfo.user, "repl_user");
        Ok(())
    }

    #[test]
    fn advertised_postgres_port_defaults_to_listen_port() {
        let cfg = sample_runtime_config();
        assert_eq!(advertised_postgres_port(&cfg), cfg.postgres.listen_port);
    }

    #[test]
    fn advertised_postgres_port_prefers_explicit_override() {
        let cfg = crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_advertise_port(Some(6543))
            .build();
        assert_eq!(advertised_postgres_port(&cfg), 6543);
    }
}


===== src/local_physical.rs =====
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use thiserror::Error;

use crate::{
    postgres_managed_conf::{MANAGED_RECOVERY_SIGNAL_NAME, MANAGED_STANDBY_SIGNAL_NAME},
    state::{SystemIdentifier, TimelineId, Version, WalLsn},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DataDirKind {
    Missing,
    Empty,
    Initialized,
    InvalidNonEmptyWithoutPgVersion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignalFileState {
    None,
    Standby,
    Recovery,
    Conflicting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalPhysicalState {
    pub(crate) data_dir_kind: DataDirKind,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) pg_version: Option<Version>,
    pub(crate) control_file_state: Option<String>,
    pub(crate) timeline_id: Option<TimelineId>,
    pub(crate) durable_end_lsn: Option<WalLsn>,
    pub(crate) was_in_recovery: Option<bool>,
    pub(crate) signal_file_state: SignalFileState,
    pub(crate) eligible_for_bootstrap: bool,
    pub(crate) eligible_for_direct_follow: bool,
    pub(crate) eligible_for_rewind: bool,
    pub(crate) eligible_for_basebackup: bool,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum LocalPhysicalStateError {
    #[error("failed to inspect data dir `{path}`: {message}")]
    DataDirIo { path: PathBuf, message: String },
    #[error("pgdata path is not a directory: `{0}`")]
    NotDirectory(PathBuf),
    #[error("invalid PG_VERSION in `{path}`: {message}")]
    InvalidPgVersion { path: PathBuf, message: String },
    #[error("pg_controldata failed for `{data_dir}` using `{binary}`: {message}")]
    PgControlDataCommand {
        binary: PathBuf,
        data_dir: PathBuf,
        message: String,
    },
    #[error("pg_controldata output was invalid for `{data_dir}`: {message}")]
    PgControlDataParse { data_dir: PathBuf, message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedPgControlData {
    system_identifier: SystemIdentifier,
    control_file_state: String,
    timeline_id: Option<TimelineId>,
    durable_end_lsn: Option<WalLsn>,
    was_in_recovery: bool,
}

pub(crate) fn inspect_local_physical_state(
    data_dir: &Path,
    postgres_binary: &Path,
) -> Result<LocalPhysicalState, LocalPhysicalStateError> {
    let kind = inspect_data_dir_kind(data_dir)?;
    let signal_file_state = inspect_signal_file_state(data_dir)?;

    match kind {
        DataDirKind::Missing => Ok(LocalPhysicalState {
            data_dir_kind: kind,
            system_identifier: None,
            pg_version: None,
            control_file_state: None,
            timeline_id: None,
            durable_end_lsn: None,
            was_in_recovery: None,
            signal_file_state,
            eligible_for_bootstrap: true,
            eligible_for_direct_follow: false,
            eligible_for_rewind: false,
            eligible_for_basebackup: true,
        }),
        DataDirKind::Empty => Ok(LocalPhysicalState {
            data_dir_kind: kind,
            system_identifier: None,
            pg_version: None,
            control_file_state: None,
            timeline_id: None,
            durable_end_lsn: None,
            was_in_recovery: None,
            signal_file_state,
            eligible_for_bootstrap: true,
            eligible_for_direct_follow: false,
            eligible_for_rewind: false,
            eligible_for_basebackup: true,
        }),
        DataDirKind::InvalidNonEmptyWithoutPgVersion => Ok(LocalPhysicalState {
            data_dir_kind: kind,
            system_identifier: None,
            pg_version: None,
            control_file_state: None,
            timeline_id: None,
            durable_end_lsn: None,
            was_in_recovery: None,
            signal_file_state,
            eligible_for_bootstrap: false,
            eligible_for_direct_follow: false,
            eligible_for_rewind: false,
            eligible_for_basebackup: false,
        }),
        DataDirKind::Initialized => {
            let pg_version = read_pg_version(data_dir)?;
            let parsed = run_pg_controldata(data_dir, postgres_binary)?;
            let signals_conflict = signal_file_state == SignalFileState::Conflicting;
            Ok(LocalPhysicalState {
                data_dir_kind: kind,
                system_identifier: Some(parsed.system_identifier),
                pg_version: Some(pg_version),
                control_file_state: Some(parsed.control_file_state),
                timeline_id: parsed.timeline_id,
                durable_end_lsn: parsed.durable_end_lsn,
                was_in_recovery: Some(parsed.was_in_recovery),
                signal_file_state,
                eligible_for_bootstrap: false,
                eligible_for_direct_follow: !signals_conflict,
                eligible_for_rewind: !signals_conflict,
                eligible_for_basebackup: true,
            })
        }
    }
}

fn inspect_data_dir_kind(data_dir: &Path) -> Result<DataDirKind, LocalPhysicalStateError> {
    match fs::metadata(data_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(LocalPhysicalStateError::NotDirectory(data_dir.to_path_buf()));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DataDirKind::Missing);
        }
        Err(err) => {
            return Err(LocalPhysicalStateError::DataDirIo {
                path: data_dir.to_path_buf(),
                message: err.to_string(),
            });
        }
    }

    if data_dir.join("PG_VERSION").exists() {
        return Ok(DataDirKind::Initialized);
    }

    let mut entries =
        fs::read_dir(data_dir).map_err(|err| LocalPhysicalStateError::DataDirIo {
            path: data_dir.to_path_buf(),
            message: err.to_string(),
        })?;
    if entries.next().is_none() {
        Ok(DataDirKind::Empty)
    } else {
        Ok(DataDirKind::InvalidNonEmptyWithoutPgVersion)
    }
}

fn inspect_signal_file_state(data_dir: &Path) -> Result<SignalFileState, LocalPhysicalStateError> {
    let standby_present = file_exists(data_dir.join(MANAGED_STANDBY_SIGNAL_NAME).as_path())?;
    let recovery_present = file_exists(data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME).as_path())?;
    Ok(match (standby_present, recovery_present) {
        (false, false) => SignalFileState::None,
        (true, false) => SignalFileState::Standby,
        (false, true) => SignalFileState::Recovery,
        (true, true) => SignalFileState::Conflicting,
    })
}

fn file_exists(path: &Path) -> Result<bool, LocalPhysicalStateError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(LocalPhysicalStateError::DataDirIo {
            path: path.to_path_buf(),
            message: err.to_string(),
        }),
    }
}

fn read_pg_version(data_dir: &Path) -> Result<Version, LocalPhysicalStateError> {
    let path = data_dir.join("PG_VERSION");
    let raw = fs::read_to_string(&path).map_err(|err| LocalPhysicalStateError::DataDirIo {
        path: path.clone(),
        message: err.to_string(),
    })?;
    let trimmed = raw.trim();
    let parsed = trimmed
        .parse::<u64>()
        .map_err(|err| LocalPhysicalStateError::InvalidPgVersion {
            path,
            message: err.to_string(),
        })?;
    Ok(Version(parsed))
}

fn run_pg_controldata(
    data_dir: &Path,
    postgres_binary: &Path,
) -> Result<ParsedPgControlData, LocalPhysicalStateError> {
    let binary = derived_pg_controldata_path(postgres_binary);
    let output = Command::new(&binary)
        .arg(data_dir)
        .output()
        .map_err(|err| LocalPhysicalStateError::PgControlDataCommand {
            binary: binary.clone(),
            data_dir: data_dir.to_path_buf(),
            message: err.to_string(),
        })?;
    if !output.status.success() {
        return Err(LocalPhysicalStateError::PgControlDataCommand {
            binary,
            data_dir: data_dir.to_path_buf(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    let stdout = String::from_utf8(output.stdout).map_err(|err| {
        LocalPhysicalStateError::PgControlDataParse {
            data_dir: data_dir.to_path_buf(),
            message: err.to_string(),
        }
    })?;
    parse_pg_controldata_output(stdout.as_str()).map_err(|message| {
        LocalPhysicalStateError::PgControlDataParse {
            data_dir: data_dir.to_path_buf(),
            message,
        }
    })
}

fn derived_pg_controldata_path(postgres_binary: &Path) -> PathBuf {
    match postgres_binary.parent() {
        Some(parent) => {
            let sibling = parent.join("pg_controldata");
            if sibling.is_file() {
                sibling
            } else {
                PathBuf::from("pg_controldata")
            }
        }
        None => PathBuf::from("pg_controldata"),
    }
}

fn parse_pg_controldata_output(output: &str) -> Result<ParsedPgControlData, String> {
    let key_values = output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let system_identifier = key_values
        .get("Database system identifier")
        .ok_or_else(|| "missing Database system identifier".to_string())
        .and_then(|value| parse_u64_field(value, "Database system identifier"))
        .map(SystemIdentifier)?;

    let control_file_state = key_values
        .get("Database cluster state")
        .cloned()
        .ok_or_else(|| "missing Database cluster state".to_string())?;

    let checkpoint_timeline = key_values
        .get("Latest checkpoint's TimeLineID")
        .map(|value| parse_u32_field(value, "Latest checkpoint's TimeLineID"))
        .transpose()?
        .map(TimelineId);
    let recovery_timeline = key_values
        .get("Min recovery ending loc's timeline")
        .map(|value| parse_u32_field(value, "Min recovery ending loc's timeline"))
        .transpose()?
        .map(TimelineId);

    let durable_end_lsn = key_values
        .get("Minimum recovery ending location")
        .or_else(|| key_values.get("Latest checkpoint location"))
        .map(|value| parse_wal_lsn(value))
        .transpose()?;

    Ok(ParsedPgControlData {
        system_identifier,
        control_file_state: control_file_state.clone(),
        timeline_id: checkpoint_timeline.or(recovery_timeline),
        durable_end_lsn,
        was_in_recovery: control_file_state.contains("recovery"),
    })
}

fn parse_u64_field(raw: &str, field: &str) -> Result<u64, String> {
    raw.parse::<u64>()
        .map_err(|err| format!("invalid {field}: {err}"))
}

fn parse_u32_field(raw: &str, field: &str) -> Result<u32, String> {
    raw.parse::<u32>()
        .map_err(|err| format!("invalid {field}: {err}"))
}

fn parse_wal_lsn(raw: &str) -> Result<WalLsn, String> {
    let Some((high, low)) = raw.split_once('/') else {
        return Err(format!("invalid WAL LSN `{raw}`"));
    };
    let high = u64::from_str_radix(high, 16)
        .map_err(|err| format!("invalid WAL LSN `{raw}` high bits: {err}"))?;
    let low = u64::from_str_radix(low, 16)
        .map_err(|err| format!("invalid WAL LSN `{raw}` low bits: {err}"))?;
    Ok(WalLsn((high << 32) | low))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        inspect_local_physical_state, parse_pg_controldata_output, DataDirKind, SignalFileState,
    };
    use crate::{
        postgres_managed_conf::{MANAGED_RECOVERY_SIGNAL_NAME, MANAGED_STANDBY_SIGNAL_NAME},
        state::{SystemIdentifier, TimelineId, WalLsn},
    };

    fn temp_dir(label: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("pgtm-local-physical-{label}-{unique}"));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[test]
    fn parses_pg_controldata_output() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = parse_pg_controldata_output(
            "\
Database system identifier:           7374392058184458932
Database cluster state:               shut down in recovery
Latest checkpoint location:           0/16B6C50
Latest checkpoint's TimeLineID:       4
Minimum recovery ending location:     0/16B6D00
Min recovery ending loc's timeline:   4
",
        )?;
        assert_eq!(parsed.system_identifier, SystemIdentifier(7_374_392_058_184_458_932));
        assert_eq!(parsed.timeline_id, Some(TimelineId(4)));
        assert_eq!(parsed.durable_end_lsn, Some(WalLsn(0x00000000_016B6D00)));
        assert!(parsed.was_in_recovery);
        Ok(())
    }

    #[test]
    fn classifies_invalid_non_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir("invalid-non-empty")?;
        fs::write(dir.join("orphan"), "data")?;
        let inspected = inspect_local_physical_state(&dir, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.data_dir_kind, DataDirKind::InvalidNonEmptyWithoutPgVersion);
        assert_eq!(inspected.signal_file_state, SignalFileState::None);
        assert!(!inspected.eligible_for_bootstrap);
        Ok(())
    }

    #[test]
    fn classifies_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir("empty")?;
        let inspected = inspect_local_physical_state(&dir, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.data_dir_kind, DataDirKind::Empty);
        assert!(inspected.eligible_for_bootstrap);
        Ok(())
    }

    #[test]
    fn detects_conflicting_signal_files_without_pg_controldata() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = temp_dir("conflicting-signals")?;
        fs::write(dir.join("orphan"), "data")?;
        fs::write(dir.join(MANAGED_STANDBY_SIGNAL_NAME), "")?;
        fs::write(dir.join(MANAGED_RECOVERY_SIGNAL_NAME), "")?;
        let inspected = inspect_local_physical_state(&dir, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.signal_file_state, SignalFileState::Conflicting);
        Ok(())
    }

    #[test]
    fn missing_directory_is_bootstrap_eligible() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir("missing-root")?;
        let missing = dir.join("missing");
        let inspected = inspect_local_physical_state(&missing, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.data_dir_kind, DataDirKind::Missing);
        assert!(inspected.eligible_for_bootstrap);
        assert_eq!(inspected.pg_version, None);
        Ok(())
    }
}


===== src/ha/process_dispatch.rs =====
use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    dcs::state::MemberRecord,
    ha::decision::HaDecision,
    local_physical::{inspect_local_physical_state, DataDirKind, SignalFileState},
    postgres_managed_conf::{
        managed_standby_auth_from_role_auth, render_managed_primary_conninfo,
        ManagedPostgresStartIntent, ManagedStandbyAuth, MANAGED_POSTGRESQL_CONF_NAME,
    },
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, PromoteSpec,
            ShutdownMode, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    pginfo::state::PgInfoState,
    state::{JobId, MemberId},
};

use super::{
    actions::{ActionId, HaAction},
    source_conn::{basebackup_source_from_member, rewind_source_from_member},
    state::HaWorkerCtx,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessDispatchOutcome {
    Applied,
    Skipped,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("remote source selection failed for action `{action:?}`: {message}")]
    SourceSelection { action: ActionId, message: String },
    #[error("process dispatch does not support action `{action:?}`")]
    UnsupportedAction { action: ActionId },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    match action {
        HaAction::AcquireLeaderLease | HaAction::ReleaseLeaderLease | HaAction::ClearSwitchover => {
            Err(ProcessDispatchError::UnsupportedAction {
                action: action.id(),
            })
        }
        HaAction::StartPostgres => {
            let leader_member_id = start_postgres_leader_member_id(ctx);
            if postgres_data_dir_requires_basebackup(
                &ctx.process_defaults.postgres_binary,
                runtime_config.postgres.data_dir.as_path(),
            )? {
                if let Some(leader_member_id) = leader_member_id {
                    let source = validate_basebackup_source(ctx, action.id(), leader_member_id)?;
                    let request = ProcessJobRequest {
                        id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                        kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                            data_dir: runtime_config.postgres.data_dir.clone(),
                            source,
                            timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                        }),
                    };
                    send_process_request(ctx, action.id(), request)?;
                    return Ok(ProcessDispatchOutcome::Applied);
                }
            }
            let start_intent = managed_start_intent_from_dcs(
                ctx,
                action.id(),
                leader_member_id,
                runtime_config.postgres.data_dir.as_path(),
            )?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.id(),
                message: err.to_string(),
            })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    config_file: managed.postgresql_conf_path,
                    log_file: ctx.process_defaults.log_file.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FollowLeader { leader_member_id } => {
            let leader_member_id = MemberId(leader_member_id.clone());
            let start_intent = managed_start_intent_from_dcs(
                ctx,
                action.id(),
                Some(&leader_member_id),
                runtime_config.postgres.data_dir.as_path(),
            )?;
            if follow_leader_is_already_current_or_pending(
                ctx,
                action.id(),
                runtime_config.postgres.data_dir.as_path(),
                &start_intent,
            )? {
                return Ok(ProcessDispatchOutcome::Skipped);
            }
            let _managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.id(),
                message: err.to_string(),
            })?;
            let demote_request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    timeout_ms: None,
                    mode: ctx.process_defaults.shutdown_mode.clone(),
                }),
            };
            send_process_request(ctx, action.id(), demote_request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::PromoteToPrimary => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Promote(PromoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::DemoteToReplica => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ctx.process_defaults.shutdown_mode.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartRewind { leader_member_id } => {
            let source = validate_rewind_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::PgRewind(PgRewindSpec {
                    target_data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartBaseBackup { leader_member_id } => {
            let source = validate_basebackup_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::RunBootstrap => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    superuser_username: runtime_config.postgres.roles.superuser.username.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FenceNode => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Fencing(FencingSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ShutdownMode::Immediate,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::WipeDataDir => {
            wipe_data_dir(runtime_config.postgres.data_dir.as_path()).map_err(|message| {
                ProcessDispatchError::Filesystem {
                    action: action.id(),
                    message,
                }
            })?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::SignalFailSafe => Ok(ProcessDispatchOutcome::Skipped),
    }
}

pub(crate) fn validate_rewind_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::RewinderSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    rewind_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

pub(crate) fn validate_basebackup_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::ReplicatorSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    basebackup_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

fn resolve_source_member(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<MemberRecord, ProcessDispatchError> {
    let dcs = ctx.dcs_subscriber.latest();
    dcs.value
        .cache
        .members
        .get(leader_member_id)
        .cloned()
        .ok_or_else(|| ProcessDispatchError::SourceSelection {
            action,
            message: format!(
                "target member `{}` not present in DCS cache",
                leader_member_id.0
            ),
        })
}

fn send_process_request(
    ctx: &mut HaWorkerCtx,
    action: ActionId,
    request: ProcessJobRequest,
) -> Result<(), ProcessDispatchError> {
    ctx.process_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action,
            message: err.to_string(),
        })
}

fn start_postgres_leader_member_id(ctx: &HaWorkerCtx) -> Option<&MemberId> {
    match &ctx.state.decision {
        HaDecision::WaitForPostgres {
            leader_member_id, ..
        } => leader_member_id.as_ref(),
        _ => None,
    }
}

fn managed_start_intent_from_dcs(
    ctx: &HaWorkerCtx,
    action: ActionId,
    replica_leader_member_id: Option<&MemberId>,
    data_dir: &Path,
) -> Result<ManagedPostgresStartIntent, ProcessDispatchError> {
    if let Some(leader_member_id) = replica_leader_member_id {
        let leader = resolve_source_member(ctx, action.clone(), leader_member_id)?;
        let source = basebackup_source_from_member(&ctx.self_id, &leader, &ctx.process_defaults)
            .map_err(|err| ProcessDispatchError::SourceSelection {
                action: action.clone(),
                message: err.to_string(),
            })?;
        return Ok(ManagedPostgresStartIntent::replica(
            source.conninfo.clone(),
            managed_standby_auth_from_role_auth(&source.auth, data_dir),
            None,
        ));
    }

    let inspected = inspect_local_physical_state(data_dir, &ctx.process_defaults.postgres_binary)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
            action: action.clone(),
            message: err.to_string(),
        })?;

    if inspected.signal_file_state != SignalFileState::None {
        return Err(ProcessDispatchError::ManagedConfig {
            action,
            message:
                "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config"
                    .to_string(),
        });
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn postgres_data_dir_requires_basebackup(
    postgres_binary: &Path,
    data_dir: &Path,
) -> Result<bool, ProcessDispatchError> {
    let inspected = inspect_local_physical_state(data_dir, postgres_binary).map_err(|err| {
        ProcessDispatchError::Filesystem {
            action: ActionId::StartPostgres,
            message: err.to_string(),
        }
    })?;
    Ok(!matches!(inspected.data_dir_kind, DataDirKind::Initialized))
}

fn follow_leader_is_already_current_or_pending(
    ctx: &HaWorkerCtx,
    action: ActionId,
    data_dir: &Path,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<bool, ProcessDispatchError> {
    let Some((expected_primary_conninfo, _)) = standby_start_details(start_intent) else {
        return Ok(true);
    };

    let pg = ctx.pg_subscriber.latest();
    let Some(current_primary_conninfo) = current_primary_conninfo(&pg.value) else {
        return Ok(true);
    };
    if current_primary_conninfo.host == expected_primary_conninfo.host
        && current_primary_conninfo.port == expected_primary_conninfo.port
    {
        return Ok(true);
    }
    if pginfo_common(&pg.value).sql == crate::pginfo::state::SqlStatus::Healthy {
        return Ok(false);
    }

    managed_config_already_targets_start_intent(action, data_dir, start_intent)
}

fn pginfo_common(state: &PgInfoState) -> &crate::pginfo::state::PgInfoCommon {
    match state {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common,
    }
}

fn standby_start_details(
    start_intent: &ManagedPostgresStartIntent,
) -> Option<(&crate::pginfo::state::PgConnInfo, &ManagedStandbyAuth)> {
    match start_intent {
        ManagedPostgresStartIntent::Replica {
            primary_conninfo,
            standby_auth,
            ..
        }
        | ManagedPostgresStartIntent::Recovery {
            primary_conninfo,
            standby_auth,
            ..
        } => Some((primary_conninfo, standby_auth)),
        ManagedPostgresStartIntent::Primary => None,
    }
}

fn managed_config_already_targets_start_intent(
    action: ActionId,
    data_dir: &Path,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<bool, ProcessDispatchError> {
    let Some((expected_primary_conninfo, standby_auth)) = standby_start_details(start_intent) else {
        return Ok(false);
    };
    let managed_conf_path = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
    let rendered = match fs::read_to_string(&managed_conf_path) {
        Ok(rendered) => rendered,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => {
            return Err(ProcessDispatchError::ManagedConfig {
                action,
                message: format!(
                    "read managed postgres config failed at {}: {err}",
                    managed_conf_path.display()
                ),
            });
        }
    };
    let expected_recovery_state = start_intent.recovery_signal();
    let actual_recovery_state =
        crate::postgres_managed::inspect_managed_recovery_state(data_dir).map_err(|err| {
            ProcessDispatchError::ManagedConfig {
                action: action.clone(),
                message: err.to_string(),
            }
        })?;
    if actual_recovery_state != expected_recovery_state {
        return Ok(false);
    }

    Ok(rendered.contains(
        render_managed_primary_conninfo(expected_primary_conninfo, standby_auth).as_str(),
    ))
}

fn current_primary_conninfo(state: &PgInfoState) -> Option<&crate::pginfo::state::PgConnInfo> {
    match state {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common.pg_config.primary_conninfo.as_ref(),
    }
}

fn process_job_id(
    scope: &str,
    self_id: &crate::state::MemberId,
    action: &HaAction,
    index: usize,
    tick: u64,
) -> JobId {
    JobId(format!(
        "ha-{}-{}-{}-{}-{}",
        scope.trim_matches('/'),
        self_id.0,
        tick,
        index,
        action.id().label(),
    ))
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), String> {
    if data_dir.as_os_str().is_empty() {
        return Err("wipe_data_dir data_dir must not be empty".to_string());
    }
    if data_dir.exists() {
        fs::remove_dir_all(data_dir)
            .map_err(|err| format!("wipe_data_dir remove_dir_all failed: {err}"))?;
    }
    fs::create_dir_all(data_dir)
        .map_err(|err| format!("wipe_data_dir create_dir_all failed: {err}"))?;
    set_postgres_data_dir_permissions(data_dir)?;
    Ok(())
}

fn set_postgres_data_dir_permissions(data_dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))
            .map_err(|err| format!("wipe_data_dir set_permissions failed: {err}"))?;
    }

    #[cfg(not(unix))]
    {
        let _ = data_dir;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        config::{RoleAuthConfig, RuntimeConfig, SecretSource},
        dcs::{
            state::{DcsCache, DcsState, DcsTrust, MemberRecord, MemberRole},
            store::{DcsLeaderStore, DcsStore, DcsStoreError, WatchEvent},
        },
        ha::{
            actions::HaAction,
            decision::HaDecision,
            process_dispatch::{
                dispatch_process_action, managed_start_intent_from_dcs, ProcessDispatchError,
                ProcessDispatchOutcome,
            },
            state::{HaState, HaWorkerContractStubInputs, HaWorkerCtx},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        postgres_managed_conf::managed_standby_auth_from_role_auth,
        process::state::{ProcessJobKind, ProcessState},
        state::{new_state_channel, MemberId, UnixMillis, WorkerError, WorkerStatus},
    };

    #[derive(Default)]
    struct NoopStore;

    impl DcsStore for NoopStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn put_path_if_absent(
            &mut self,
            _path: &str,
            _value: String,
        ) -> Result<bool, DcsStoreError> {
            Ok(true)
        }

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    impl DcsLeaderStore for NoopStore {
        fn acquire_leader_lease(
            &mut self,
            _scope: &str,
            _member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn release_leader_lease(
            &mut self,
            _scope: &str,
            _member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn clear_switchover(&mut self, _scope: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }
    }

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: SecretSource::Inline {
                content: "secret-password".to_string(),
            },
        }
    }

    fn unique_test_data_dir(label: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let sequence = TEST_DATA_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-process-dispatch-{label}-{}-{millis}-{sequence}",
            std::process::id(),
        ))
    }

    fn sample_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        }
    }

    fn sample_dcs_state(config: RuntimeConfig) -> DcsState {
        DcsState {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FreshQuorum,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config,
                cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn build_context(
        runtime_config: RuntimeConfig,
    ) -> (
        HaWorkerCtx,
        crate::state::StatePublisher<PgInfoState>,
        crate::state::StatePublisher<DcsState>,
        tokio::sync::mpsc::UnboundedReceiver<crate::process::state::ProcessJobRequest>,
    ) {
        let (config_publisher, config_subscriber) =
            new_state_channel(runtime_config.clone(), UnixMillis(1));
        let (pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(runtime_config.clone()), UnixMillis(1));
        let (process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (ha_publisher, _ha_subscriber) = new_state_channel(
            HaState {
                worker: WorkerStatus::Starting,
                phase: crate::ha::state::HaPhase::Init,
                tick: 0,
                decision: crate::ha::decision::HaDecision::NoChange,
            },
            UnixMillis(1),
        );
        let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();

        let _ = config_publisher;
        let _ = pg_publisher;
        let _ = dcs_publisher;
        let _ = process_publisher;

        (
            HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
                publisher: ha_publisher,
                config_subscriber,
                pg_subscriber,
                dcs_subscriber,
                process_subscriber,
                process_inbox: process_tx,
                dcs_store: Box::new(NoopStore),
                scope: "scope-a".to_string(),
                self_id: MemberId("node-a".to_string()),
            }),
            pg_publisher,
            dcs_publisher,
            process_rx,
        )
    }

    fn primary_member(member_id: &str, host: &str, port: u16) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: host.to_string(),
            postgres_port: port,
            api_url: None,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
            updated_at: UnixMillis(1),
            pg_version: crate::state::Version(1),
        }
    }

    fn replica_pg_state(primary_host: &str, primary_port: u16) -> PgInfoState {
        replica_pg_state_with_sql(primary_host, primary_port, SqlStatus::Healthy)
    }

    fn replica_pg_state_with_sql(
        primary_host: &str,
        primary_port: u16,
        sql: SqlStatus,
    ) -> PgInfoState {
        PgInfoState::Replica {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: sql.clone(),
                readiness: match sql {
                    SqlStatus::Healthy => Readiness::Ready,
                    SqlStatus::Unknown => Readiness::Unknown,
                    SqlStatus::Unreachable => Readiness::NotReady,
                },
                timeline: None,
                pg_config: PgConfig {
                    port: Some(5432),
                    hot_standby: Some(true),
                    primary_conninfo: Some(crate::pginfo::state::PgConnInfo {
                        host: primary_host.to_string(),
                        port: primary_port,
                        user: "replicator".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: Some(5),
                        ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
                        options: None,
                    }),
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
            replay_lsn: crate::state::WalLsn(10),
            follow_lsn: Some(crate::state::WalLsn(11)),
            upstream: None,
        }
    }

    fn remove_dir_if_present(path: &PathBuf) -> Result<(), WorkerError> {
        if path.exists() {
            fs::remove_dir_all(path)
                .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        }
        Ok(())
    }

    fn initdb_data_dir(runtime_config: &RuntimeConfig, data_dir: &Path) -> Result<(), WorkerError> {
        let output = std::process::Command::new(&runtime_config.process.binaries.initdb)
            .arg("-D")
            .arg(data_dir)
            .arg("-U")
            .arg("postgres")
            .arg("--auth=trust")
            .arg("--no-sync")
            .env("LC_ALL", "C")
            .output()
            .map_err(|err| WorkerError::Message(format!("initdb fixture failed: {err}")))?;
        if !output.status.success() {
            return Err(WorkerError::Message(format!(
                "initdb fixture failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_builds_request_with_managed_settings() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, _dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        assert_eq!(
            request.id.0,
            "ha-scope-a-node-a-7-3-start_postgres".to_string()
        );
        if let ProcessJobKind::StartPostgres(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(
                spec.config_file,
                runtime_config
                    .postgres
                    .data_dir
                    .join("pgtm.postgresql.conf")
            );
        } else {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_uses_basebackup_when_data_dir_is_missing_and_leader_is_known(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-missing-data-dir");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.20", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: Some(MemberId("node-b".to_string())),
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if let ProcessJobKind::BaseBackup(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.source.conninfo.host, "10.0.0.20".to_string());
            assert_eq!(spec.source.conninfo.port, 5440);
        } else {
            return Err(WorkerError::Message(
                "expected basebackup request when data dir is missing".to_string(),
            ));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_preserves_replica_follow_target() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-replica");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        initdb_data_dir(&runtime_config, &data_dir)?;
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.20", 5432),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: Some(MemberId("node-b".to_string())),
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if !matches!(request.kind, ProcessJobKind::StartPostgres(_)) {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        let managed_conf_path = runtime_config
            .postgres
            .data_dir
            .join("pgtm.postgresql.conf");
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            WorkerError::Message(format!(
                "read managed postgres conf failed at {}: {err}",
                managed_conf_path.display()
            ))
        })?;
        if !rendered.contains("primary_conninfo") {
            return Err(WorkerError::Message(format!(
                "expected replica managed config to include primary_conninfo, got:\n{rendered}"
            )));
        }
        if !rendered.contains("passfile=") {
            return Err(WorkerError::Message(format!(
                "expected replica managed config to include managed passfile, got:\n{rendered}"
            )));
        }
        let standby_signal = runtime_config.postgres.data_dir.join("standby.signal");
        if !standby_signal.exists() {
            return Err(WorkerError::Message(format!(
                "expected standby.signal to exist at {}",
                standby_signal.display()
            )));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_without_replica_target_starts_primary() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-primary");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.20", 5432),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: None,
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if !matches!(request.kind, ProcessJobKind::StartPostgres(_)) {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        let managed_conf_path = runtime_config
            .postgres
            .data_dir
            .join("pgtm.postgresql.conf");
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            WorkerError::Message(format!(
                "read managed postgres conf failed at {}: {err}",
                managed_conf_path.display()
            ))
        })?;
        if rendered.contains("primary_conninfo") {
            return Err(WorkerError::Message(format!(
                "expected primary managed config without primary_conninfo, got:\n{rendered}"
            )));
        }
        let standby_signal = runtime_config.postgres.data_dir.join("standby.signal");
        if standby_signal.exists() {
            return Err(WorkerError::Message(format!(
                "expected standby.signal to be absent at {}",
                standby_signal.display()
            )));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_rejects_existing_replica_state_without_dcs_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-existing-replica-without-leader");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, _dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let existing_conninfo = crate::pginfo::state::PgConnInfo {
            host: "10.0.0.20".to_string(),
            port: 5432,
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: Some(2),
            ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
            options: Some("-c wal_receiver_status_interval=5s".to_string()),
        };
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &crate::postgres_managed_conf::ManagedPostgresStartIntent::replica(
                existing_conninfo.clone(),
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    runtime_config.postgres.data_dir.as_path(),
                ),
                Some("slot_a".to_string()),
            ),
        )
        .map_err(|err| {
            WorkerError::Message(format!("seed managed replica config failed: {err}"))
        })?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: None,
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert!(matches!(
            outcome,
            Err(ProcessDispatchError::ManagedConfig { .. })
        ));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_rewrites_managed_config_and_demotes_when_upstream_changes(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-reload");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        pg_publisher
            .publish(replica_pg_state("10.0.0.20", 5432), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            7,
            3,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        assert_eq!(
            request.id.0,
            "ha-scope-a-node-a-7-3-follow_leader_node-b".to_string()
        );
        if let ProcessJobKind::Demote(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.mode, ctx.process_defaults.shutdown_mode);
        } else {
            return Err(WorkerError::Message(
                "expected demote request".to_string(),
            ));
        }
        assert!(process_rx.try_recv().is_err());

        let managed_conf_path = runtime_config
            .postgres
            .data_dir
            .join("pgtm.postgresql.conf");
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            WorkerError::Message(format!(
                "read managed postgres conf failed at {}: {err}",
                managed_conf_path.display()
            ))
        })?;
        if !rendered.contains("10.0.0.21") || !rendered.contains("5440") {
            return Err(WorkerError::Message(format!(
                "expected managed config to retarget node-b, got:\n{rendered}"
            )));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_skips_when_upstream_already_matches_authoritative_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-steady-state");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        pg_publisher
            .publish(replica_pg_state("10.0.0.21", 5440), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            7,
            3,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Skipped));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_applies_when_live_upstream_mismatches_even_if_managed_config_already_targets_authoritative_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-pending-retarget");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &crate::postgres_managed_conf::ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.21".to_string(),
                    port: 5440,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
                    options: Some("-c wal_receiver_status_interval=5s".to_string()),
                },
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    runtime_config.postgres.data_dir.as_path(),
                ),
                None,
            ),
        )
        .map_err(|err| {
            WorkerError::Message(format!("seed managed replica config failed: {err}"))
        })?;
        let follow_action = HaAction::FollowLeader {
            leader_member_id: "node-b".to_string(),
        };
        let retargeted_start_intent = managed_start_intent_from_dcs(
            &ctx,
            follow_action.id(),
            Some(&leader_member_id),
            runtime_config.postgres.data_dir.as_path(),
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "derive leader-backed start intent for seeded managed config failed: {err}"
            ))
        })?;
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &retargeted_start_intent,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "rewrite managed replica config to authoritative state failed: {err}"
            ))
        })?;
        pg_publisher
            .publish(replica_pg_state("10.0.0.20", 5432), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(&mut ctx, 7, 3, &follow_action, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));
        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        assert_eq!(
            request.id.0,
            "ha-scope-a-node-a-7-3-follow_leader_node-b".to_string()
        );
        if let ProcessJobKind::Demote(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.mode, ctx.process_defaults.shutdown_mode);
        } else {
            return Err(WorkerError::Message(
                "expected demote request".to_string(),
            ));
        }
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_skips_when_managed_config_already_targets_authoritative_leader_and_pginfo_is_not_healthy(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-pending-restart");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        let follow_action = HaAction::FollowLeader {
            leader_member_id: "node-b".to_string(),
        };
        let retargeted_start_intent = managed_start_intent_from_dcs(
            &ctx,
            follow_action.id(),
            Some(&leader_member_id),
            runtime_config.postgres.data_dir.as_path(),
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "derive leader-backed start intent for seeded managed config failed: {err}"
            ))
        })?;
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &retargeted_start_intent,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "rewrite managed replica config to authoritative state failed: {err}"
            ))
        })?;
        pg_publisher
            .publish(
                replica_pg_state_with_sql(
                    "10.0.0.20",
                    5432,
                    crate::pginfo::state::SqlStatus::Unreachable,
                ),
                UnixMillis(2),
            )
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(&mut ctx, 7, 3, &follow_action, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Skipped));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_skips_until_pginfo_reports_primary_conninfo(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-no-primary-conninfo-yet");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            7,
            3,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Skipped));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn wipe_data_dir_dispatch_recreates_directory() -> Result<(), WorkerError> {
        let base_dir = std::env::temp_dir().join(format!(
            "pgtuskmaster-process-dispatch-{}",
            std::process::id()
        ));
        let nested_file = base_dir.join("stale.txt");
        if base_dir.exists() {
            fs::remove_dir_all(&base_dir).map_err(|err| {
                WorkerError::Message(format!("cleanup existing temp dir failed: {err}"))
            })?;
        }
        fs::create_dir_all(&base_dir)
            .and_then(|()| fs::write(&nested_file, b"stale"))
            .map_err(|err| {
                WorkerError::Message(format!("create temp dir fixture failed: {err}"))
            })?;

        let runtime_config = sample_runtime_config(base_dir.clone());
        let (mut ctx, _pg_publisher, _dcs_publisher, _process_rx) =
            build_context(runtime_config.clone());
        let outcome =
            dispatch_process_action(&mut ctx, 2, 0, &HaAction::WipeDataDir, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));
        assert!(base_dir.exists());
        assert!(!nested_file.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(&base_dir)
                .map_err(|err| {
                    WorkerError::Message(format!("read recreated data dir metadata failed: {err}"))
                })?
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o700);
        }

        fs::remove_dir_all(&base_dir)
            .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        Ok(())
    }

    #[test]
    fn start_basebackup_dispatch_uses_target_member_endpoint_and_replicator_role(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("basebackup");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let mut dcs_state = sample_dcs_state(runtime_config.clone());
        dcs_state.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.20", 5440),
        );
        let _ = dcs_publisher
            .publish(dcs_state, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs state failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            9,
            0,
            &HaAction::StartBaseBackup {
                leader_member_id: leader_member_id.clone(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if let ProcessJobKind::BaseBackup(spec) = request.kind {
            assert_eq!(spec.source.conninfo.host, "10.0.0.20".to_string());
            assert_eq!(spec.source.conninfo.port, 5440);
            assert_eq!(spec.source.conninfo.user, "replicator".to_string());
            assert_eq!(spec.source.auth, sample_password_auth());
        } else {
            return Err(WorkerError::Message(
                "expected basebackup request".to_string(),
            ));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_rewind_dispatch_uses_target_member_and_ignores_unrelated_leader_key(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("rewind");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let unrelated_leader_id = MemberId("node-c".to_string());
        let mut dcs_state = sample_dcs_state(runtime_config.clone());
        dcs_state.cache.leader = Some(crate::dcs::state::LeaderRecord {
            member_id: unrelated_leader_id.clone(),
        });
        dcs_state.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.21", 5441),
        );
        dcs_state.cache.members.insert(
            unrelated_leader_id.clone(),
            primary_member("node-c", "10.0.0.99", 5999),
        );
        let _ = dcs_publisher
            .publish(dcs_state, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs state failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            10,
            0,
            &HaAction::StartRewind {
                leader_member_id: leader_member_id.clone(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if let ProcessJobKind::PgRewind(spec) = request.kind {
            assert_eq!(spec.source.conninfo.host, "10.0.0.21".to_string());
            assert_eq!(spec.source.conninfo.port, 5441);
            assert_eq!(spec.source.conninfo.user, "rewinder".to_string());
            assert_eq!(spec.source.auth, sample_password_auth());
        } else {
            return Err(WorkerError::Message("expected rewind request".to_string()));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_basebackup_dispatch_rejects_missing_target_member() {
        let data_dir = unique_test_data_dir("missing-member");
        let runtime_config = sample_runtime_config(data_dir);
        let (mut ctx, _pg_publisher, _dcs_publisher, _process_rx) =
            build_context(runtime_config.clone());

        let outcome = dispatch_process_action(
            &mut ctx,
            11,
            0,
            &HaAction::StartBaseBackup {
                leader_member_id: MemberId("node-missing".to_string()),
            },
            &runtime_config,
        );

        assert!(matches!(
            outcome,
            Err(ProcessDispatchError::SourceSelection { .. })
        ));
    }
}
