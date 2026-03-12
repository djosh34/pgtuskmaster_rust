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

docs/src/tutorial/validating-cluster-behavior.md

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
docs/src/reference/debug-api.md
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

# Reference

- [Reference](reference/overview.md)
    - [HTTP API](reference/http-api.md)
    - [HA Decisions](reference/ha-decisions.md)
    - [Debug API](reference/debug-api.md)
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
cucumber = "0.22.1"
futures = "0.3.31"
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
src/cli/switchover.rs
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
src/ha/reconcile.rs
src/ha/source_conn.rs
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
src/pginfo/state.rs
src/pginfo/worker.rs
src/postgres_managed.rs
src/postgres_managed_conf.rs
src/postgres_roles.rs
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
src/test_harness/mod.rs
src/test_harness/namespace.rs
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
tests/docker/Dockerfile
tests/docker/entrypoint.sh
tests/docker/wrappers/pg_basebackup
tests/docker/wrappers/pg_rewind
tests/docker/wrappers/postgres
tests/ha.rs
tests/ha/features/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins.feature
tests/ha/features/ha_basebackup_clone_blocked_then_unblocked_replica_recovers/ha_basebackup_clone_blocked_then_unblocked_replica_recovers.feature
tests/ha/features/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum/ha_broken_replica_rejoin_attempt_does_not_destabilize_quorum.feature
tests/ha/features/ha_dcs_and_api_faults_then_healed_cluster_converges/ha_dcs_and_api_faults_then_healed_cluster_converges.feature
tests/ha/features/ha_dcs_quorum_lost_enters_failsafe/ha_dcs_quorum_lost_enters_failsafe.feature
tests/ha/features/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes/ha_dcs_quorum_lost_fencing_blocks_post_cutoff_writes.feature
tests/ha/features/ha_lagging_replica_is_not_promoted_during_failover/ha_lagging_replica_is_not_promoted_during_failover.feature
tests/ha/features/ha_non_primary_api_isolated_primary_stays_primary/ha_non_primary_api_isolated_primary_stays_primary.feature
tests/ha/features/ha_old_primary_partitioned_from_majority_majority_elects_new_primary/ha_old_primary_partitioned_from_majority_majority_elects_new_primary.feature
tests/ha/features/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover/ha_old_primary_partitioned_then_healed_rejoins_as_replica_after_majority_failover.feature
tests/ha/features/ha_planned_switchover_changes_primary_cleanly/ha_planned_switchover_changes_primary_cleanly.feature
tests/ha/features/ha_planned_switchover_with_concurrent_writes/ha_planned_switchover_with_concurrent_writes.feature
tests/ha/features/ha_primary_killed_custom_roles_survive_rejoin/ha_primary_killed_custom_roles_survive_rejoin.feature
tests/ha/features/ha_primary_killed_then_rejoins_as_replica/ha_primary_killed_then_rejoins_as_replica.feature
tests/ha/features/ha_primary_killed_with_concurrent_writes/ha_primary_killed_with_concurrent_writes.feature
tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature
tests/ha/features/ha_repeated_failovers_preserve_single_primary/ha_repeated_failovers_preserve_single_primary.feature
tests/ha/features/ha_replica_flapped_primary_stays_primary/ha_replica_flapped_primary_stays_primary.feature
tests/ha/features/ha_replica_partitioned_from_majority_primary_stays_primary/ha_replica_partitioned_from_majority_primary_stays_primary.feature
tests/ha/features/ha_replica_stopped_primary_stays_primary/ha_replica_stopped_primary_stays_primary.feature
tests/ha/features/ha_replication_path_isolated_then_healed_replicas_catch_up/ha_replication_path_isolated_then_healed_replicas_catch_up.feature
tests/ha/features/ha_rewind_fails_then_basebackup_rejoins_old_primary/ha_rewind_fails_then_basebackup_rejoins_old_primary.feature
tests/ha/features/ha_targeted_switchover_promotes_requested_replica/ha_targeted_switchover_promotes_requested_replica.feature
tests/ha/features/ha_targeted_switchover_to_degraded_replica_is_rejected/ha_targeted_switchover_to_degraded_replica_is_rejected.feature
tests/ha/features/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken/ha_two_nodes_stopped_then_one_healthy_node_restarted_restores_service_while_other_stays_broken.feature
tests/ha/features/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum/ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum.feature
tests/ha/givens/three_node_custom_roles/compose.yml
tests/ha/givens/three_node_custom_roles/configs/node-a/runtime.toml
tests/ha/givens/three_node_custom_roles/configs/node-b/runtime.toml
tests/ha/givens/three_node_custom_roles/configs/node-c/runtime.toml
tests/ha/givens/three_node_custom_roles/configs/observer/node-a.toml
tests/ha/givens/three_node_custom_roles/configs/observer/node-b.toml
tests/ha/givens/three_node_custom_roles/configs/observer/node-c.toml
tests/ha/givens/three_node_custom_roles/configs/pg_hba.conf
tests/ha/givens/three_node_custom_roles/configs/pg_ident.conf
tests/ha/givens/three_node_custom_roles/configs/tls/ca.crt
tests/ha/givens/three_node_custom_roles/configs/tls/node-a.crt
tests/ha/givens/three_node_custom_roles/configs/tls/node-a.key
tests/ha/givens/three_node_custom_roles/configs/tls/node-b.crt
tests/ha/givens/three_node_custom_roles/configs/tls/node-b.key
tests/ha/givens/three_node_custom_roles/configs/tls/node-c.crt
tests/ha/givens/three_node_custom_roles/configs/tls/node-c.key
tests/ha/givens/three_node_custom_roles/configs/tls/observer.crt
tests/ha/givens/three_node_custom_roles/configs/tls/observer.key
tests/ha/givens/three_node_custom_roles/secrets/api-admin-token
tests/ha/givens/three_node_custom_roles/secrets/api-read-token
tests/ha/givens/three_node_custom_roles/secrets/postgres-superuser-password
tests/ha/givens/three_node_custom_roles/secrets/replicator-password
tests/ha/givens/three_node_custom_roles/secrets/rewinder-password
tests/ha/givens/three_node_plain/compose.yml
tests/ha/givens/three_node_plain/configs/node-a/runtime.toml
tests/ha/givens/three_node_plain/configs/node-b/runtime.toml
tests/ha/givens/three_node_plain/configs/node-c/runtime.toml
tests/ha/givens/three_node_plain/configs/observer/node-a.toml
tests/ha/givens/three_node_plain/configs/observer/node-b.toml
tests/ha/givens/three_node_plain/configs/observer/node-c.toml
tests/ha/givens/three_node_plain/configs/pg_hba.conf
tests/ha/givens/three_node_plain/configs/pg_ident.conf
tests/ha/givens/three_node_plain/configs/tls/ca.crt
tests/ha/givens/three_node_plain/configs/tls/node-a.crt
tests/ha/givens/three_node_plain/configs/tls/node-a.key
tests/ha/givens/three_node_plain/configs/tls/node-b.crt
tests/ha/givens/three_node_plain/configs/tls/node-b.key
tests/ha/givens/three_node_plain/configs/tls/node-c.crt
tests/ha/givens/three_node_plain/configs/tls/node-c.key
tests/ha/givens/three_node_plain/configs/tls/observer.crt
tests/ha/givens/three_node_plain/configs/tls/observer.key
tests/ha/givens/three_node_plain/secrets/api-admin-token
tests/ha/givens/three_node_plain/secrets/api-read-token
tests/ha/givens/three_node_plain/secrets/postgres-superuser-password
tests/ha/givens/three_node_plain/secrets/replicator-password
tests/ha/givens/three_node_plain/secrets/rewinder-password
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
tests/ha/support/workload/mod.rs
tests/ha/support/world/mod.rs
tests/nextest_config_contract.rs


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
docs/draft/docs/src/how-to/handle-complex-failures.md
docs/draft/docs/src/how-to/handle-complex-failures.revised.md
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
docs/draft/docs/src/tutorial/validating-cluster-behavior.md
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
docs/src/reference/debug-api.md
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
docs/tmp/docs/src/how-to/handle-complex-failures.prompt.md
docs/tmp/docs/src/reference/ha-decisions.prompt.md
docs/tmp/docs/src/tutorial/validating-cluster-behavior.prompt.md
docs/tmp/ha-replica-stopped-primary-stays-primary-output.txt
docs/tmp/verbose_extra_context/handle-complex-failures-context.md
docs/tmp/verbose_extra_context/validating-cluster-behavior-context.md


===== tests/ha/features/ha_primary_killed_then_rejoins_as_replica/ha_primary_killed_then_rejoins_as_replica.feature =====
Feature: ha_primary_killed_then_rejoins_as_replica
  Scenario: a killed primary fails over and later rejoins as a replica
    Given the "three_node_plain" harness is running
    And the cluster reaches one stable primary
    When the current primary container crashes
    Then after the configured HA lease deadline a different node becomes the only primary
    And I can write a proof row through the new primary
    When I start the killed node container again
    Then after the configured recovery deadline the restarted node rejoins as a replica
    And the proof row is visible from the restarted node
    And the cluster still has exactly one primary


===== tests/ha/support/observer/mod.rs =====
pub mod pgtm;
pub mod sql;


===== docs/src/tutorial/observing-failover.md =====
# Observing a Failover Event

In this tutorial, you will watch a real failover unfold in a three-node cluster. Start with the cluster-wide `pgtm` view, then inspect one node more deeply only when you need the retained history.

## Prerequisites

Complete the [First HA Cluster](first-ha-cluster.md) tutorial and keep the cluster running.

Use these docs-owned operator configs while you follow the event:

- [`docs/examples/docker-cluster-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-a.toml)
- [`docs/examples/docker-cluster-node-b.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-b.toml)
- [`docs/examples/docker-cluster-node-c.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-c.toml)

Each file mirrors the shipped docker runtime config and adds `[pgtm].api_url` for the corresponding host-mapped API port.

## Step 1: Check initial cluster health

Start from one seed node:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

Initially:

- trust should be healthy
- exactly one member should appear as `primary`
- the `DEBUG` column should be `available` on nodes with debug enabled
- the detail block should show `decision=no_change`

## Step 2: Capture a deeper baseline on the current leader

Pick the current leader from the status output. Then inspect that node directly by using the matching docs example config:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml debug verbose
```

If another node is leader, swap in `docker-cluster-node-b.toml` or `docker-cluster-node-c.toml`.

If you want to archive the exact payload before the fault, save the JSON form:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml --json debug verbose > before-failover.json
```

## Step 3: Introduce a primary outage

Create a failure in the current primary using the fault method available in your environment. The source-backed test harness exercises both immediate PostgreSQL stops and network faults created through proxy links.

## Step 4: Watch trust and leadership move

Use repeated cluster-wide status while the fault is active:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v --watch
```

Watch for:

- trust degradation
- changes in `LEADER`
- one node moving toward candidacy or primary behavior
- `DECISION` changing from wait states into leadership actions

## Step 5: Inspect the election on the winning node

Once a likely winner appears, inspect that node directly:

```bash
pgtm -c docs/examples/docker-cluster-node-b.toml debug verbose
```

During the election window, you should see the decision evolve roughly like this:

1. `wait_for_dcs_trust`
2. `attempt_leadership`
3. `become_primary`
4. `no_change`

The retained `changes` and `timeline` sections are the fastest way to reconstruct the order of those transitions.

## Step 6: Verify the new leader

After promotion completes, run the cluster view again:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

At this point you should see:

- a new leader
- one member reporting `primary`
- the previously failed member showing degraded SQL health until it recovers
- trust returning to `full_quorum` once the cluster stabilizes

## Step 7: Restore the failed node

Restore the failed node using the inverse of the fault you introduced.

## Step 8: Watch node recovery

Keep watching the cluster summary and, when needed, inspect the recovering node:

```bash
pgtm -c docs/examples/docker-cluster-node-c.toml debug verbose
```

The recovering node may temporarily show `recover_replica` behavior. Once caught up, it should return to replica behavior with healthy SQL state.

## Step 9: Final verification

Run one final cluster summary:

```bash
pgtm -c docs/examples/docker-cluster-node-a.toml status -v
```

Confirm:

- all expected members are present
- exactly one primary remains
- trust is back to `full_quorum`
- the cluster has returned to steady-state decisions

## What you learned

You have now observed a complete failover cycle: detection, election, promotion, and recovery. `pgtm status -v` gives the cluster-wide picture, and `pgtm debug verbose` gives the single-node retained history behind it.


===== docs/src/how-to/handle-primary-failure.md =====
# Handle Primary Failure

This guide shows how to detect, assess, and respond to a PostgreSQL primary node failure with `pgtm` as the primary operator interface.

## Prerequisites

- a running pgtuskmaster cluster with more than one node
- access to at least one reachable node API
- access to PostgreSQL on the cluster nodes
- an operator-facing config that either sets `[pgtm].api_url` or derives an operator-reachable URL from `api.listen_addr`

## Detect primary failure

Start with the cluster-wide view:

```bash
pgtm -c config.toml status -v
```

Focus on:

- `ROLE`
- `TRUST`
- `PHASE`
- `LEADER`
- `DECISION`
- warning lines about unreachable peers or disagreement

A primary failure usually surfaces as:

- no stable primary in the sampled view
- trust degrading to `fail_safe` or `not_trusted`
- one or more nodes moving through `candidate_leader`, `rewinding`, or other recovery-related phases

If the cluster view is degraded, repeat the same command from another seed config before you conclude the failure scope.

## Assess cluster state

Look for:

- exactly one node in `ROLE=primary` once the cluster settles
- all other nodes in replica-oriented behavior
- `TRUST=full_quorum` on the healthy view

The Docker cluster example uses `ha.lease_ttl_ms = 10000` and `ha.loop_interval_ms = 1000`. Those values bound member freshness checks and also define the etcd leader-lease TTL. In abrupt-node-loss cases, the old leader is invalidated when etcd expires that lease and the watched DCS cache drops `/{scope}/leader`.

If the status table is not enough, inspect the most suspicious node directly:

```bash
pgtm -c config.toml debug verbose
```

That gives you the current:

- PostgreSQL state
- DCS trust and leader cache
- HA phase and decision
- retained `changes` and `timeline`

## Respond to primary failure

No manual intervention is required for most failures. The HA decision engine automatically:

1. Detects PostgreSQL unreachability.
2. Releases the local leader lease when the primary can still step down cleanly.
3. Otherwise waits for etcd to expire the dead primary's lease-backed leader key.
4. Moves through recovery and election logic on the surviving majority.
5. Selects a recovery target from healthy members.
6. Executes rewind or base-backup recovery so the failed node can rejoin safely.

To monitor automated recovery, keep watching cluster status:

```bash
pgtm -c config.toml status -v --watch
```

### If automation stalls

If decisions remain unchanged and trust stays at `fail_safe` or `not_trusted`, resolve the underlying etcd, network, or PostgreSQL problem before expecting promotion to proceed. A healthy 2-of-3 majority should not remain stuck behind a dead primary's stale leader metadata once the old leader lease has actually expired.

## Verify recovery

Once a new primary is visible in cluster status, verify data consistency:

1. Confirm exactly one primary via SQL:

   ```bash
   for node in node-a node-b node-c; do
     psql -h "${node}" -U postgres -c "SELECT pg_is_in_recovery();"
   done
   ```

   Exactly one node should return `false`.

2. Check replication lag on replicas:

   ```bash
   psql -h <replica-ip> -U postgres -c "SELECT now() - pg_last_xact_replay_timestamp();"
   ```

3. Confirm `pgtm status -v` has returned to one primary, healthy trust, and no warning lines.

## Troubleshoot common scenarios

### All nodes show `TRUST=fail_safe`

Cause: etcd unreachable or most member records stale.  
Action: restore etcd cluster health first.

### New primary elected but replicas stay in recovery work

Cause: rewind or catch-up still running, or recovery escalated because WAL continuity was not sufficient.  
Action: keep watching `status -v` and inspect `debug verbose` on the affected replica.

### You suspect split-brain

Cause: network partition or stale observations.  
Action: run `pgtm status -v` from more than one seed config and treat any sustained multi-primary view as critical.

### Leader lease release stalls

Cause: the previous primary cannot reach etcd to revoke its own lease cleanly.  
Action: wait for lease expiry on the etcd side, then verify that `LEADER` clears from `pgtm status -v` and that the surviving majority returns to `TRUST=full_quorum`. `ha.lease_ttl_ms` bounds both member freshness and the leader-lease TTL.

## Verification checklist

- [ ] Exactly one node reports `ROLE=primary`
- [ ] All other nodes report replica-oriented behavior
- [ ] `TRUST=full_quorum` on the healthy view
- [ ] `pg_is_in_recovery()` returns `false` on one node only
- [ ] Replication lag on replicas is acceptable
- [ ] No sustained split-brain evidence appears in repeated status samples
- [ ] Application traffic can write to the new primary


===== docs/src/explanation/failure-modes.md =====
# Failure modes and recovery behavior

This page explains how pgtuskmaster responds to component failures. It covers the system's trust model, how failures are categorized, and the reasoning behind recovery strategies. Understanding these concepts helps operators predict system behavior during outages and make informed decisions about deployment topology and configuration.

## The DCS trust model

pgtuskmaster's behavior depends heavily on its view of cluster state, which comes from a distributed configuration store (DCS). The system does not treat DCS as either fully reliable or fully unreliable. Instead, it evaluates trust continuously and makes distinct decisions at each trust level.

### Trust levels

The system uses three discrete trust evaluations:

**FullQuorum**
The DCS is healthy and at least two members have fresh metadata. The system can safely perform leader elections, coordinate switchovers, and enforce split-brain prevention.

**FailSafe**
The DCS is accessible but does not meet full consensus requirements. This occurs when the local member record is stale or fewer than two members appear fresh in a multi-member view. In this state the system limits its activity to prevent data corruption.

**NotTrusted**
The DCS is unreachable or otherwise unhealthy. All trust-dependent operations are suspended.

### Why trust degrades

Trust degrades to protect against split-brain scenarios. If a node cannot verify that its view of the cluster is current, acting on stale information could cause it to promote itself while another primary is still active. The system prefers to pause or enter a safe mode rather than risk data divergence.

Trust evaluation follows a specific sequence:

1. If etcd itself reports unhealthy, trust becomes `NotTrusted`
2. If the local member record is missing or older than `ha.lease_ttl_ms`, trust becomes `FailSafe`
3. In clusters larger than one node, if fewer than two members have fresh records, trust becomes `FailSafe`
4. Only when all checks pass does trust become `FullQuorum`

This design reflects a key principle: membership metadata freshness acts as a heartbeat. A node that stops updating its record is treated as failed, even if the DCS remains healthy.

Leader liveness is lease-backed rather than inferred from stale metadata. The etcd store attaches `/{scope}/leader` to an etcd lease derived from `ha.lease_ttl_ms`. If the owner releases leadership, it revokes its own lease. If the owner dies hard, keepalive stops and etcd deletes the leader key automatically when the lease expires. The watch-fed DCS cache then removes the leader record, allowing a healthy majority to continue election without manual DCS cleanup.

## PostgreSQL reachability as a distinct axis

While DCS trust affects coordination safety, PostgreSQL reachability determines what local actions are possible. The system treats these as orthogonal concerns. A node can have `FullQuorum` trust while its local PostgreSQL is unreachable, or vice versa.

PostgreSQL reachability is binary in decision logic: either `SqlStatus::Healthy` or not. `Unknown` and `Unreachable` states both block replication and promotion actions. This binary approach simplifies state management but has important implications for recovery behavior.

## Failure classification and phase transitions

When failures occur, the system transitions through specific HA phases. Each phase represents a coherent state where the system waits for a condition or performs a bounded set of actions.

### Initial failure response

The decision logic in `src/ha/decide.rs` prioritizes safety over availability. If DCS trust is not `FullQuorum`, the system immediately routes to `FailSafe` phase. The only exception is when the local PostgreSQL is a confirmed healthy primary, in which case it emits `EnterFailSafe` to ensure the leader lease is released.

This behavior ensures that network partitions or DCS outages do not create split-brain scenarios. By entering `FailSafe`, nodes avoid taking coordinated actions until they can verify cluster state.

### Primary failure handling

When a primary node fails, the recovery sequence depends on whether the failure is detected internally (postgres stops) or externally (DCS marks it stale).

**Internal detection (postgres becomes unreachable):**
If the node holds the leader lease, it releases its lease with reason `PostgresUnreachable` and transitions to `Rewinding`. This signals other nodes that the primary is stepping down intentionally.

**External detection (other nodes observe failure):**
When replicas observe that the old leader lease has expired and no active leader remains in DCS, they follow standard leader election. A replica transitions from `Replica` to `CandidateLeader`, attempts to acquire the leader lease, and promotes to primary if successful.

The `Rewinding` phase is intentional: it provides a dedicated state where the node reconciles its potentially diverged state before rejoining as a replica. This prevents a former primary from immediately following a new leader without first rewinding or re-cloning.

### Replica failure handling

Replica failure follows a simpler path. If PostgreSQL becomes unreachable, the replica enters `WaitingPostgresReachable` and periodically attempts to start it. The allowed source set supports that waiting behavior and the `WaitForPostgres` decision, but not a stronger claim about a separate timeout-based escalation policy for prolonged outages.

## Recovery mechanisms

The system supports three recovery strategies, each with specific use cases and safety implications.

### Rewind recovery

Rewind uses `pg_rewind` to reconcile a diverged former primary with its new upstream. This is efficient because it only transfers changed blocks. The decision engine emits `StartRewind` when a timeline divergence is detected.

The engine detects divergence by comparing timelines: if the local timeline does not match the leader's timeline, rewind is required. This check prevents unnecessary rewind operations when timelines are already consistent.

### Base backup recovery

When rewind is not possible or fails, the system falls back to base backup. This performs a full physical copy from the primary. The decision engine emits `StartBaseBackup` after rewind failure or when no local timeline exists.

Base backup is slower and more resource-intensive than rewind.

### Bootstrap recovery

Bootstrap creates a new cluster from scratch. This is used only during initial cluster formation, not for recovery. The distinction is important: bootstrap assumes an empty data directory, while recovery assumes a potentially corrupted or diverged existing directory.

## Safety mechanisms and split-brain prevention

The system prevents split-brain through a combination of leader leases, fencing, and explicit phase constraints.

### Leader leases

A leader lease is a DCS entry that a primary must hold to be considered authoritative. Acquiring the lease requires a DCS write that succeeds only if no other node holds it. Releasing the lease is a deliberate action that triggers specific downstream behaviors.

In the etcd-backed store, the leader key is attached to an etcd lease. When a primary detects it should step down (switchover or external leader detection), it revokes its own lease before demoting. If the process dies hard, the missing keepalive causes etcd to expire the lease and delete the key automatically. This ensures that no node can rely on a blind delete of another node's leader key.

### Fencing

Fencing is the process of forcibly stopping a misbehaving primary. The system enters `Fencing` phase when it detects an apparent split-brain: local PostgreSQL is primary but DCS shows a different leader.

The fencing process runs as an independent job. Success transitions back to `WaitingDcsTrusted` with a lease release. Failure transitions to `FailSafe`, halting all further action. This conservative approach reflects that fencing failure indicates deeper infrastructure problems.

### Observer-based split-brain detection

The test harness includes an `HaInvariantObserver` that samples cluster state and immediately fails if it observes two primaries simultaneously. This is not part of the production runtime but validates the design: the system must never allow dual-primary scenarios in observable windows.

The observer's existence demonstrates that split-brain prevention is a first-class design goal, not an afterthought. It also shows how operators can implement similar monitoring in production.

## Fail-safe mode

`FailSafe` is the system's panic mode. It is not a recovery state but a holding pattern. Unlike other phases, `FailSafe` does not automatically attempt recovery. It persists until DCS trust is restored, at which point it exits to `WaitingDcsTrusted`.

The rationale is that entering `FailSafe` indicates insufficient information to make safe decisions. Automated recovery would risk exacerbating an unknown failure mode. Human operators must investigate and restore trust conditions.

The system may emit `SignalFailSafe` to local processes.

## Timeout behavior and missing source support

The source code shows several timeout mechanisms but does not expose operator-configurable retry policies or maximum outage durations before escalation. For example:

- etcd commands have a hard-coded 2-second timeout
- Process jobs have deadlines but the decision engine does not automatically escalate after repeated timeouts
- The HA loop polls at a configured interval but does not implement backoff

Missing source support for specific retry counts and escalation timers means the safest statement is simply that the code exposes timeouts and deadlines, but the provided source set does not prove a richer operator-facing escalation policy.

The source-backed behavior is intentionally conservative: degraded trust routes to `FailSafe`, primary loss can release leadership and move through rewind or base-backup recovery, and fencing exists to handle foreign-leader detection.


===== docs/tmp/verbose_extra_context/validating-cluster-behavior-context.md =====
# Verbose Context For `validating-cluster-behavior`

This tutorial target sits between the existing "observing failover" tutorial and the operator-facing how-to pages for handling failures.

The key teaching goal is not cluster setup and not incident response procedure. The teaching goal is validation:

- bring up a development HA cluster,
- trigger a concrete failure,
- watch the operator-visible surfaces change,
- confirm that PostgreSQL connectivity and replica behavior match the expected HA contract,
- and then repeat the observation loop for another fault pattern.

The most relevant current source material for this is the HA cucumber suite under `tests/ha/features/`.

Important current product behavior that should stay factually aligned with the tutorial:

- The operator-visible `pgtm primary` view now fails closed when sampling is too weak to trust a primary, but it still resolves a primary when there is a unique sampled primary with enough corroborating observation.
- Generic planned switchovers are now resolved to a specific target at request time, so a requested switchover should converge to one chosen replica instead of being re-resolved repeatedly by different nodes.
- A storage-stalled primary is kept fenced even after it drops the leader lease, so a stalled old primary should not drift back into ordinary no-lease behavior during failover.
- The HA validation harness now explicitly tracks proof-convergence blockers for `pg_basebackup` blockage and PostgreSQL-path isolation, which means failure scenarios intentionally testing lag or recovery do not incorrectly wait for full proof-row convergence too early.

For a tutorial, the most useful operator-visible observations are:

- `pgtm status --json` to inspect sampled members, roles, health, and warnings.
- `pgtm primary --tls --json` to confirm which node is the current writable primary from the operator perspective.
- `pgtm replicas --tls --json` to confirm which nodes are currently treated as replicas.
- proof-row checks through SQL to confirm that writes remain visible on the expected nodes before and after failover or recovery.

The HA cucumber scenarios show a few especially teachable validation patterns:

- Primary killed then rejoins as replica:
  This is the cleanest failover-and-rejoin exercise. It demonstrates a stable failover, a new primary becoming authoritative, and the old primary rejoining safely as a replica.
- Replica stopped while primary stays primary:
  This is a simpler "negative" validation. The important learning point is that not every fault should produce failover. The cluster should keep the same primary and keep serving writes while the stopped replica later rejoins.
- Planned switchover:
  This is the cleanest planned control-plane handoff exercise. It teaches users to distinguish a controlled role change from an unplanned failure response.

If K2 wants to make the tutorial narrower, the safest tutorial arc is:

1. Confirm a healthy cluster and identify the primary.
2. Record an initial proof row.
3. Stop a replica and validate that the primary does not change.
4. Restart the replica and confirm it rejoins and catches up.
5. Optionally repeat with a primary-failure scenario if K2 wants a second exercise section.

If K2 wants a richer tutorial, it can sequence two exercises:

1. A no-failover validation where a replica outage keeps the current primary stable.
2. A failover validation where the primary is killed and the old primary later rejoins as a replica.

The runtime evidence file `docs/tmp/ha-replica-stopped-primary-stays-primary-output.txt` comes from a passing ultra-long scenario run on the current tree and can be cited for exact step names and successful outcome text if K2 wants concrete phrasing grounded in the current harness behavior.


===== docs/tmp/ha-replica-stopped-primary-stays-primary-output.txt =====
usage: make test-long [TESTS="ha_test_one"|TESTS="ha_test_one ha_test_two"]
make[1]: Entering directory '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust'
cargo nextest run --workspace --profile ultra-long --no-tests fail --target-dir "/tmp/pgtuskmaster_rust-target" --config "build.incremental=false" --test ha -- ha_replica_stopped_primary_stays_primary --exact
make[1]: Leaving directory '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust'
make[1]: Entering directory '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust'
python3 ./tools/export-nextest-junit-logs.py ./target/nextest/ultra-long/junit.xml ./target/nextest/ultra-long/logs
make[1]: Leaving directory '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust'
