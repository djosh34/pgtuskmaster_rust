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

docs/src/reference/ha-decisions.md

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
docs/tmp/verbose_extra_context/handle-complex-failures-context.md


===== src/api/mod.rs =====
use std::fmt;

use thiserror::Error;

pub(crate) mod controller;
pub(crate) mod fallback;
pub mod worker;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("dcs store error: {0}")]
    DcsStore(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedResponse {
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaStateResponse {
    pub cluster_name: String,
    pub scope: String,
    pub self_member_id: String,
    pub leader: Option<String>,
    pub switchover_pending: bool,
    pub switchover_to: Option<String>,
    pub member_count: usize,
    pub members: Vec<HaClusterMemberResponse>,
    pub dcs_trust: DcsTrustResponse,
    pub authority: HaAuthorityResponse,
    pub fence_cutoff: Option<FenceCutoffResponse>,
    pub ha_role: TargetRoleResponse,
    pub ha_tick: u64,
    pub planned_actions: Vec<ReconcileActionResponse>,
    pub snapshot_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaClusterMemberResponse {
    pub member_id: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub api_url: Option<String>,
    pub role: MemberRoleResponse,
    pub sql: SqlStatusResponse,
    pub readiness: ReadinessResponse,
    pub timeline: Option<u64>,
    pub write_lsn: Option<u64>,
    pub replay_lsn: Option<u64>,
    pub updated_at_ms: u64,
    pub pg_version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsTrustResponse {
    FullQuorum,
    FailSafe,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HaAuthorityResponse {
    Primary { member_id: String, epoch: LeaseEpochResponse },
    NoPrimary { reason: NoPrimaryReasonResponse },
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseEpochResponse {
    pub holder: String,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FenceCutoffResponse {
    pub epoch: LeaseEpochResponse,
    pub committed_lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoPrimaryReasonResponse {
    DcsDegraded,
    LeaseOpen,
    Recovering,
    SwitchoverRejected { blocker: SwitchoverBlockerResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SwitchoverBlockerResponse {
    TargetMissing,
    TargetIneligible { reason: IneligibleReasonResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TargetRoleResponse {
    Leader { epoch: LeaseEpochResponse },
    Candidate { candidacy: CandidacyResponse },
    Follower { goal: FollowGoalResponse },
    FailSafe { goal: FailSafeGoalResponse },
    DemotingForSwitchover { member_id: String },
    Fenced { reason: FenceReasonResponse },
    Idle { reason: IdleReasonResponse },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CandidacyResponse {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover { member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowGoalResponse {
    pub leader: String,
    pub recovery: RecoveryPlanResponse,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryPlanResponse {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FailSafeGoalResponse {
    PrimaryMustStop { cutoff: FenceCutoffResponse },
    ReplicaKeepFollowing { upstream: Option<String> },
    WaitForQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdleReasonResponse {
    AwaitingLeader,
    AwaitingTarget { member_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FenceReasonResponse {
    ForeignLeaderDetected,
    StorageStalled,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReconcileActionResponse {
    InitDb,
    BaseBackup { member_id: String },
    PgRewind { member_id: String },
    StartPrimary,
    StartReplica { member_id: String },
    Promote,
    Demote { mode: ShutdownModeResponse },
    AcquireLease { candidacy: CandidacyResponse },
    ReleaseLease,
    Publish { publication: HaAuthorityResponse },
    ClearSwitchover,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownModeResponse {
    Fast,
    Immediate,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IneligibleReasonResponse {
    NotReady,
    Lagging,
    Partitioned,
    ApiUnavailable,
    StartingUp,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRoleResponse {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SqlStatusResponse {
    Unknown,
    Healthy,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessResponse {
    Unknown,
    Ready,
    NotReady,
}

impl DcsTrustResponse {
    fn as_str(&self) -> &'static str {
        match self {
            Self::FullQuorum => "full_quorum",
            Self::FailSafe => "fail_safe",
            Self::NotTrusted => "not_trusted",
        }
    }
}

impl fmt::Display for DcsTrustResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


===== src/api/controller.rs =====
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, CandidacyResponse, DcsTrustResponse,
        FailSafeGoalResponse, FenceCutoffResponse, FenceReasonResponse, FollowGoalResponse,
        HaAuthorityResponse, HaClusterMemberResponse, HaStateResponse, IdleReasonResponse,
        IneligibleReasonResponse, LeaseEpochResponse, MemberRoleResponse, NoPrimaryReasonResponse,
        ReadinessResponse, ReconcileActionResponse, RecoveryPlanResponse, ShutdownModeResponse,
        SqlStatusResponse, SwitchoverBlockerResponse, TargetRoleResponse,
    },
    dcs::{
        state::{member_record_is_fresh, DcsTrust, MemberRecord, MemberRole, SwitchoverRequest},
        store::DcsStore,
    },
    debug_api::snapshot::SystemSnapshot,
    ha::types::{
        Candidacy, FailSafeGoal, FenceCutoff, FenceReason, FollowGoal, IdleReason,
        IneligibleReason, NoPrimaryReason, PublicationGoal, ReconcileAction, ShutdownMode,
        SwitchoverBlocker, TargetRole,
    },
    state::Versioned,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    #[serde(default)]
    pub(crate) switchover_to: Option<String>,
}

pub(crate) fn post_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    let request = validate_switchover_request(snapshot, input)?;
    let encoded = serde_json::to_string(&request)
        .map_err(|err| ApiError::internal(format!("switchover encode failed: {err}")))?;

    let path = format!("/{}/switchover", scope.trim_matches('/'));
    store
        .write_path(&path, encoded)
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;

    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn delete_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
) -> ApiResult<AcceptedResponse> {
    store
        .delete_path(format!("/{}/switchover", scope.trim_matches('/')).as_str())
        .map_err(|err| ApiError::DcsStore(err.to_string()))?;
    Ok(AcceptedResponse { accepted: true })
}

pub(crate) fn get_ha_state(snapshot: &Versioned<SystemSnapshot>) -> HaStateResponse {
    HaStateResponse {
        cluster_name: snapshot.value.config.value.cluster.name.clone(),
        scope: snapshot.value.config.value.dcs.scope.clone(),
        self_member_id: snapshot.value.config.value.cluster.member_id.clone(),
        leader: snapshot
            .value
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        switchover_pending: snapshot.value.dcs.value.cache.switchover.is_some(),
        switchover_to: snapshot
            .value
            .dcs
            .value
            .cache
            .switchover
            .as_ref()
            .and_then(|request| request.switchover_to.as_ref().map(|member_id| member_id.0.clone())),
        member_count: snapshot.value.dcs.value.cache.members.len(),
        members: snapshot
            .value
            .dcs
            .value
            .cache
            .members
            .values()
            .map(map_member_record)
            .collect(),
        dcs_trust: map_dcs_trust(&snapshot.value.dcs.value.trust),
        authority: map_authority(&snapshot.value.ha.value.publication.authority),
        fence_cutoff: snapshot
            .value
            .ha
            .value
            .publication
            .fence_cutoff
            .as_ref()
            .map(map_fence_cutoff),
        ha_role: map_target_role(&snapshot.value.ha.value.role),
        ha_tick: snapshot.value.ha.value.tick,
        planned_actions: snapshot
            .value
            .ha
            .value
            .planned_actions
            .iter()
            .map(map_action)
            .collect(),
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn validate_switchover_request(
    snapshot: Option<&SystemSnapshot>,
    input: SwitchoverRequestInput,
) -> ApiResult<SwitchoverRequest> {
    let Some(raw_target) = input.switchover_to else {
        return Ok(SwitchoverRequest {
            switchover_to: None,
        });
    };
    let snapshot =
        snapshot.ok_or_else(|| ApiError::DcsStore("snapshot unavailable".to_string()))?;

    let target = raw_target.trim();
    if target.is_empty() {
        return Err(ApiError::bad_request(
            "switchover_to must not be empty".to_string(),
        ));
    }

    let target_member_id = crate::state::MemberId(target.to_string());
    let members = &snapshot.dcs.value.cache.members;
    let target_member = members
        .get(&target_member_id)
        .ok_or_else(|| ApiError::bad_request(format!("unknown switchover_to member `{target}`")))?;

    if snapshot
        .dcs
        .value
        .cache
        .leader
        .as_ref()
        .map(|leader| leader.member_id == target_member_id)
        .unwrap_or(false)
    {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is already the leader"
        )));
    }

    let now = crate::process::worker::system_now_unix_millis()
        .map_err(|err| ApiError::internal(format!("switchover current-time read failed: {err}")))?;
    if !member_record_is_fresh(target_member, &snapshot.dcs.value.cache, now)
        || target_member.role != MemberRole::Replica
        || target_member.sql != crate::pginfo::state::SqlStatus::Healthy
        || target_member.readiness != crate::pginfo::state::Readiness::Ready
    {
        return Err(ApiError::bad_request(format!(
            "switchover_to member `{target}` is not an eligible switchover target"
        )));
    }

    Ok(SwitchoverRequest {
        switchover_to: Some(target_member_id),
    })
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FullQuorum => DcsTrustResponse::FullQuorum,
        DcsTrust::FailSafe => DcsTrustResponse::FailSafe,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
    }
}

fn map_member_record(value: &MemberRecord) -> HaClusterMemberResponse {
    HaClusterMemberResponse {
        member_id: value.member_id.0.clone(),
        postgres_host: value.postgres_host.clone(),
        postgres_port: value.postgres_port,
        api_url: value.api_url.clone(),
        role: map_member_role(&value.role),
        sql: map_sql_status(&value.sql),
        readiness: map_readiness(&value.readiness),
        timeline: value.timeline.map(|timeline| u64::from(timeline.0)),
        write_lsn: value.write_lsn.map(|lsn| lsn.0),
        replay_lsn: value.replay_lsn.map(|lsn| lsn.0),
        updated_at_ms: value.updated_at.0,
        pg_version: value.pg_version.0,
    }
}

fn map_authority(value: &crate::ha::types::AuthorityView) -> HaAuthorityResponse {
    match value {
        crate::ha::types::AuthorityView::Primary { member, epoch } => HaAuthorityResponse::Primary {
            member_id: member.0.clone(),
            epoch: map_epoch(epoch),
        },
        crate::ha::types::AuthorityView::NoPrimary(reason) => HaAuthorityResponse::NoPrimary {
            reason: map_no_primary_reason(reason),
        },
        crate::ha::types::AuthorityView::Unknown => HaAuthorityResponse::Unknown,
    }
}

fn map_epoch(value: &crate::ha::types::LeaseEpoch) -> LeaseEpochResponse {
    LeaseEpochResponse {
        holder: value.holder.0.clone(),
        generation: value.generation,
    }
}

fn map_fence_cutoff(value: &FenceCutoff) -> FenceCutoffResponse {
    FenceCutoffResponse {
        epoch: map_epoch(&value.epoch),
        committed_lsn: value.committed_lsn,
    }
}

fn map_no_primary_reason(value: &NoPrimaryReason) -> NoPrimaryReasonResponse {
    match value {
        NoPrimaryReason::DcsDegraded => NoPrimaryReasonResponse::DcsDegraded,
        NoPrimaryReason::LeaseOpen => NoPrimaryReasonResponse::LeaseOpen,
        NoPrimaryReason::Recovering => NoPrimaryReasonResponse::Recovering,
        NoPrimaryReason::SwitchoverRejected(blocker) => NoPrimaryReasonResponse::SwitchoverRejected {
            blocker: map_switchover_blocker(blocker),
        },
    }
}

fn map_switchover_blocker(value: &SwitchoverBlocker) -> SwitchoverBlockerResponse {
    match value {
        SwitchoverBlocker::TargetMissing => SwitchoverBlockerResponse::TargetMissing,
        SwitchoverBlocker::TargetIneligible(reason) => {
            SwitchoverBlockerResponse::TargetIneligible {
                reason: map_ineligible_reason(reason),
            }
        }
    }
}

fn map_target_role(value: &TargetRole) -> TargetRoleResponse {
    match value {
        TargetRole::Leader(epoch) => TargetRoleResponse::Leader {
            epoch: map_epoch(epoch),
        },
        TargetRole::Candidate(candidacy) => TargetRoleResponse::Candidate {
            candidacy: map_candidacy(candidacy),
        },
        TargetRole::Follower(goal) => TargetRoleResponse::Follower {
            goal: map_follow_goal(goal),
        },
        TargetRole::FailSafe(goal) => TargetRoleResponse::FailSafe {
            goal: map_failsafe_goal(goal),
        },
        TargetRole::DemotingForSwitchover(member_id) => {
            TargetRoleResponse::DemotingForSwitchover {
                member_id: member_id.0.clone(),
            }
        }
        TargetRole::Fenced(reason) => TargetRoleResponse::Fenced {
            reason: map_fence_reason(reason),
        },
        TargetRole::Idle(reason) => TargetRoleResponse::Idle {
            reason: map_idle_reason(reason),
        },
    }
}

fn map_candidacy(value: &Candidacy) -> CandidacyResponse {
    match value {
        Candidacy::Bootstrap => CandidacyResponse::Bootstrap,
        Candidacy::Failover => CandidacyResponse::Failover,
        Candidacy::ResumeAfterOutage => CandidacyResponse::ResumeAfterOutage,
        Candidacy::TargetedSwitchover(member_id) => CandidacyResponse::TargetedSwitchover {
            member_id: member_id.0.clone(),
        },
    }
}

fn map_follow_goal(value: &FollowGoal) -> FollowGoalResponse {
    FollowGoalResponse {
        leader: value.leader.0.clone(),
        recovery: map_recovery_plan(&value.recovery),
    }
}

fn map_recovery_plan(value: &crate::ha::types::RecoveryPlan) -> RecoveryPlanResponse {
    match value {
        crate::ha::types::RecoveryPlan::None => RecoveryPlanResponse::None,
        crate::ha::types::RecoveryPlan::StartStreaming => RecoveryPlanResponse::StartStreaming,
        crate::ha::types::RecoveryPlan::Rewind => RecoveryPlanResponse::Rewind,
        crate::ha::types::RecoveryPlan::Basebackup => RecoveryPlanResponse::Basebackup,
    }
}

fn map_failsafe_goal(value: &FailSafeGoal) -> FailSafeGoalResponse {
    match value {
        FailSafeGoal::PrimaryMustStop(cutoff) => FailSafeGoalResponse::PrimaryMustStop {
            cutoff: map_fence_cutoff(cutoff),
        },
        FailSafeGoal::ReplicaKeepFollowing(upstream) => FailSafeGoalResponse::ReplicaKeepFollowing {
            upstream: upstream.as_ref().map(|member_id| member_id.0.clone()),
        },
        FailSafeGoal::WaitForQuorum => FailSafeGoalResponse::WaitForQuorum,
    }
}

fn map_idle_reason(value: &IdleReason) -> IdleReasonResponse {
    match value {
        IdleReason::AwaitingLeader => IdleReasonResponse::AwaitingLeader,
        IdleReason::AwaitingTarget(member_id) => IdleReasonResponse::AwaitingTarget {
            member_id: member_id.0.clone(),
        },
    }
}

fn map_fence_reason(value: &FenceReason) -> FenceReasonResponse {
    match value {
        FenceReason::ForeignLeaderDetected => FenceReasonResponse::ForeignLeaderDetected,
        FenceReason::StorageStalled => FenceReasonResponse::StorageStalled,
    }
}

fn map_action(value: &ReconcileAction) -> ReconcileActionResponse {
    match value {
        ReconcileAction::InitDb => ReconcileActionResponse::InitDb,
        ReconcileAction::BaseBackup(member_id) => ReconcileActionResponse::BaseBackup {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::PgRewind(member_id) => ReconcileActionResponse::PgRewind {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::StartPrimary => ReconcileActionResponse::StartPrimary,
        ReconcileAction::StartReplica(member_id) => ReconcileActionResponse::StartReplica {
            member_id: member_id.0.clone(),
        },
        ReconcileAction::Promote => ReconcileActionResponse::Promote,
        ReconcileAction::Demote(mode) => ReconcileActionResponse::Demote {
            mode: map_shutdown_mode(*mode),
        },
        ReconcileAction::AcquireLease(candidacy) => ReconcileActionResponse::AcquireLease {
            candidacy: map_candidacy(candidacy),
        },
        ReconcileAction::ReleaseLease => ReconcileActionResponse::ReleaseLease,
        ReconcileAction::Publish(publication) => ReconcileActionResponse::Publish {
            publication: map_publication_goal(publication),
        },
        ReconcileAction::ClearSwitchover => ReconcileActionResponse::ClearSwitchover,
    }
}

fn map_publication_goal(value: &PublicationGoal) -> HaAuthorityResponse {
    match value {
        PublicationGoal::KeepCurrent => HaAuthorityResponse::Unknown,
        PublicationGoal::PublishPrimary { primary, epoch } => HaAuthorityResponse::Primary {
            member_id: primary.0.clone(),
            epoch: map_epoch(epoch),
        },
        PublicationGoal::PublishNoPrimary { reason, .. } => HaAuthorityResponse::NoPrimary {
            reason: map_no_primary_reason(reason),
        },
    }
}

fn map_shutdown_mode(value: ShutdownMode) -> ShutdownModeResponse {
    match value {
        ShutdownMode::Fast => ShutdownModeResponse::Fast,
        ShutdownMode::Immediate => ShutdownModeResponse::Immediate,
    }
}

fn map_ineligible_reason(value: &IneligibleReason) -> IneligibleReasonResponse {
    match value {
        IneligibleReason::NotReady => IneligibleReasonResponse::NotReady,
        IneligibleReason::Lagging => IneligibleReasonResponse::Lagging,
        IneligibleReason::Partitioned => IneligibleReasonResponse::Partitioned,
        IneligibleReason::ApiUnavailable => IneligibleReasonResponse::ApiUnavailable,
        IneligibleReason::StartingUp => IneligibleReasonResponse::StartingUp,
    }
}

fn map_member_role(value: &MemberRole) -> MemberRoleResponse {
    match value {
        MemberRole::Unknown => MemberRoleResponse::Unknown,
        MemberRole::Primary => MemberRoleResponse::Primary,
        MemberRole::Replica => MemberRoleResponse::Replica,
    }
}

fn map_sql_status(value: &crate::pginfo::state::SqlStatus) -> SqlStatusResponse {
    match value {
        crate::pginfo::state::SqlStatus::Unknown => SqlStatusResponse::Unknown,
        crate::pginfo::state::SqlStatus::Healthy => SqlStatusResponse::Healthy,
        crate::pginfo::state::SqlStatus::Unreachable => SqlStatusResponse::Unreachable,
    }
}

fn map_readiness(value: &crate::pginfo::state::Readiness) -> ReadinessResponse {
    match value {
        crate::pginfo::state::Readiness::Unknown => ReadinessResponse::Unknown,
        crate::pginfo::state::Readiness::Ready => ReadinessResponse::Ready,
        crate::pginfo::state::Readiness::NotReady => ReadinessResponse::NotReady,
    }
}


===== src/ha/types.rs =====
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::DcsTrust,
    process::{
        jobs::{ActiveJobKind, ShutdownMode as ProcessShutdownMode},
        state::{JobOutcome, ProcessState as WorkerProcessState},
    },
    state::{MemberId, TimelineId, UnixMillis, WalLsn},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaseEpoch {
    pub(crate) holder: MemberId,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FenceCutoff {
    pub(crate) epoch: LeaseEpoch,
    pub(crate) committed_lsn: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldView {
    pub(crate) local: LocalKnowledge,
    pub(crate) global: GlobalKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalKnowledge {
    pub(crate) data_dir: DataDirState,
    pub(crate) postgres: PostgresState,
    pub(crate) process: ProcessState,
    pub(crate) storage: StorageState,
    pub(crate) publication: PublicationState,
    pub(crate) observation: ObservationState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ObservationState {
    pub(crate) pg_observed_at: UnixMillis,
    pub(crate) last_start_success_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DataDirState {
    Missing,
    Initialized(LocalDataState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum LocalDataState {
    BootstrapEmpty,
    ConsistentReplica,
    Diverged(DivergenceState),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DivergenceState {
    RewindPossible,
    BasebackupRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PostgresState {
    Offline,
    Primary { committed_lsn: u64 },
    Replica {
        upstream: Option<MemberId>,
        replication: ReplicationState,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ReplicationState {
    Streaming(WalPosition),
    CatchingUp(WalPosition),
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ProcessState {
    Idle,
    Running(JobKind),
    Failed(JobFailure),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct JobFailure {
    pub(crate) job: JobKind,
    pub(crate) recovery: FailureRecovery,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FailureRecovery {
    RetrySameJob,
    FallbackToBasebackup,
    WaitForOperator,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum StorageState {
    Healthy,
    Stalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PublicationState {
    pub(crate) authority: AuthorityView,
    pub(crate) fence_cutoff: Option<FenceCutoff>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum AuthorityView {
    Primary {
        member: MemberId,
        epoch: LeaseEpoch,
    },
    NoPrimary(NoPrimaryReason),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlobalKnowledge {
    pub(crate) dcs_trust: DcsTrust,
    pub(crate) lease: LeaseState,
    pub(crate) observed_lease: Option<LeaseEpoch>,
    pub(crate) observed_primary: Option<MemberId>,
    pub(crate) switchover: SwitchoverState,
    pub(crate) peers: BTreeMap<MemberId, PeerKnowledge>,
    pub(crate) self_peer: PeerKnowledge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum LeaseState {
    HeldByMe(LeaseEpoch),
    HeldByPeer(LeaseEpoch),
    Unheld,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverState {
    None,
    Requested(SwitchoverRequest),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SwitchoverRequest {
    pub(crate) target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverTarget {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PeerKnowledge {
    pub(crate) election: ElectionEligibility,
    pub(crate) api: ApiVisibility,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ElectionEligibility {
    BootstrapEligible,
    PromoteEligible(WalPosition),
    Ineligible(IneligibleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum IneligibleReason {
    NotReady,
    Lagging,
    Partitioned,
    ApiUnavailable,
    StartingUp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ApiVisibility {
    Reachable,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DesiredState {
    pub(crate) role: TargetRole,
    pub(crate) publication: PublicationGoal,
    pub(crate) clear_switchover: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TargetRole {
    Leader(LeaseEpoch),
    Candidate(Candidacy),
    Follower(FollowGoal),
    FailSafe(FailSafeGoal),
    DemotingForSwitchover(MemberId),
    Fenced(FenceReason),
    Idle(IdleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Candidacy {
    Bootstrap,
    Failover,
    ResumeAfterOutage,
    TargetedSwitchover(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FollowGoal {
    pub(crate) leader: MemberId,
    pub(crate) recovery: RecoveryPlan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum RecoveryPlan {
    None,
    StartStreaming,
    Rewind,
    Basebackup,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FailSafeGoal {
    PrimaryMustStop(FenceCutoff),
    ReplicaKeepFollowing(Option<MemberId>),
    WaitForQuorum,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum IdleReason {
    AwaitingLeader,
    AwaitingTarget(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FenceReason {
    ForeignLeaderDetected,
    StorageStalled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PublicationGoal {
    KeepCurrent,
    PublishPrimary {
        primary: MemberId,
        epoch: LeaseEpoch,
    },
    PublishNoPrimary {
        reason: NoPrimaryReason,
        fence_cutoff: Option<FenceCutoff>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum NoPrimaryReason {
    DcsDegraded,
    LeaseOpen,
    Recovering,
    SwitchoverRejected(SwitchoverBlocker),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverBlocker {
    TargetMissing,
    TargetIneligible(IneligibleReason),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ReconcileAction {
    InitDb,
    BaseBackup(MemberId),
    PgRewind(MemberId),
    StartPrimary,
    StartReplica(MemberId),
    Promote,
    Demote(ShutdownMode),
    AcquireLease(Candidacy),
    ReleaseLease,
    Publish(PublicationGoal),
    ClearSwitchover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ShutdownMode {
    Fast,
    Immediate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum JobKind {
    InitDb,
    BaseBackup,
    PgRewind,
    StartPrimary,
    StartReplica,
    Promote,
    Demote,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(crate) struct WalPosition {
    pub(crate) timeline: u64,
    pub(crate) lsn: u64,
}

impl ObservationState {
    pub(crate) fn waiting_for_fresh_pg_after_start(&self) -> bool {
        self.last_start_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }
}

impl PublicationState {
    pub(crate) fn unknown() -> Self {
        Self {
            authority: AuthorityView::Unknown,
            fence_cutoff: None,
        }
    }
}

impl TargetRole {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Leader(_) => "leader",
            Self::Candidate(_) => "candidate",
            Self::Follower(_) => "follower",
            Self::FailSafe(_) => "fail_safe",
            Self::DemotingForSwitchover(_) => "demoting_for_switchover",
            Self::Fenced(_) => "fenced",
            Self::Idle(_) => "idle",
        }
    }
}

impl ReconcileAction {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::InitDb => "init_db",
            Self::BaseBackup(_) => "basebackup",
            Self::PgRewind(_) => "pg_rewind",
            Self::StartPrimary => "start_primary",
            Self::StartReplica(_) => "start_replica",
            Self::Promote => "promote",
            Self::Demote(_) => "demote",
            Self::AcquireLease(_) => "acquire_lease",
            Self::ReleaseLease => "release_lease",
            Self::Publish(_) => "publish",
            Self::ClearSwitchover => "clear_switchover",
        }
    }
}

impl ShutdownMode {
    pub(crate) fn to_process_mode(self) -> ProcessShutdownMode {
        match self {
            Self::Fast => ProcessShutdownMode::Fast,
            Self::Immediate => ProcessShutdownMode::Immediate,
        }
    }
}

impl From<&WorkerProcessState> for ProcessState {
    fn from(value: &WorkerProcessState) -> Self {
        match value {
            WorkerProcessState::Running { active, .. } => Self::Running(job_kind_from_active(
                &active.kind,
            )),
            WorkerProcessState::Idle {
                last_outcome: Some(JobOutcome::Failure { job_kind, .. }),
                ..
            }
            | WorkerProcessState::Idle {
                last_outcome: Some(JobOutcome::Timeout { job_kind, .. }),
                ..
            } => Self::Failed(JobFailure {
                job: job_kind_from_active(job_kind),
                recovery: failure_recovery_from_job(job_kind),
            }),
            WorkerProcessState::Idle { .. } => Self::Idle,
        }
    }
}

pub(crate) fn last_success_at(value: &WorkerProcessState, expected: ActiveJobKind) -> Option<UnixMillis> {
    match value {
        WorkerProcessState::Idle {
            last_outcome: Some(JobOutcome::Success {
                job_kind,
                finished_at,
                ..
            }),
            ..
        } if *job_kind == expected => Some(*finished_at),
        _ => None,
    }
}

pub(crate) fn wal_position(
    timeline: Option<TimelineId>,
    lsn: Option<WalLsn>,
) -> Option<WalPosition> {
    match (timeline, lsn) {
        (Some(timeline), Some(lsn)) => Some(WalPosition {
            timeline: u64::from(timeline.0),
            lsn: lsn.0,
        }),
        _ => None,
    }
}

fn job_kind_from_active(value: &ActiveJobKind) -> JobKind {
    match value {
        ActiveJobKind::Bootstrap => JobKind::InitDb,
        ActiveJobKind::BaseBackup => JobKind::BaseBackup,
        ActiveJobKind::PgRewind => JobKind::PgRewind,
        ActiveJobKind::Promote => JobKind::Promote,
        ActiveJobKind::Demote => JobKind::Demote,
        ActiveJobKind::StartPostgres => JobKind::StartPrimary,
    }
}

fn failure_recovery_from_job(value: &ActiveJobKind) -> FailureRecovery {
    match value {
        ActiveJobKind::PgRewind => FailureRecovery::FallbackToBasebackup,
        ActiveJobKind::BaseBackup | ActiveJobKind::Bootstrap => FailureRecovery::WaitForOperator,
        ActiveJobKind::Promote
        | ActiveJobKind::Demote
        | ActiveJobKind::StartPostgres => FailureRecovery::RetrySameJob,
    }
}


===== src/ha/decide.rs =====
use std::cmp::Ordering;

use crate::{dcs::state::DcsTrust, state::MemberId};

use super::types::{
    ApiVisibility, Candidacy, DesiredState, ElectionEligibility, FailSafeGoal, FailureRecovery,
    FenceCutoff, FenceReason, FollowGoal, IdleReason, LeaseEpoch, LeaseState, LocalDataState,
    NoPrimaryReason, PeerKnowledge, PostgresState, ProcessState, PublicationGoal, RecoveryPlan,
    StorageState, SwitchoverState, SwitchoverTarget, TargetRole, WalPosition, WorldView,
};

pub(crate) fn decide(world: &WorldView, self_id: &MemberId) -> DesiredState {
    if !matches!(world.global.dcs_trust, DcsTrust::FullQuorum) {
        return decide_degraded(world);
    }

    if world.local.storage == StorageState::Stalled {
        if let (PostgresState::Primary { committed_lsn }, Some(epoch)) =
            (&world.local.postgres, active_or_observed_epoch(world))
        {
            let cutoff = FenceCutoff {
                epoch,
                committed_lsn: *committed_lsn,
            };
            return DesiredState {
                role: TargetRole::Fenced(FenceReason::StorageStalled),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::Recovering,
                    fence_cutoff: Some(cutoff),
                },
                clear_switchover: false,
            };
        }
    }

    if let Some(epoch) = observed_foreign_lease(world, self_id) {
        let publication = PublicationGoal::PublishPrimary {
            primary: epoch.holder.clone(),
            epoch: epoch.clone(),
        };
        return match &world.local.postgres {
            PostgresState::Primary { .. } => DesiredState {
                role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
                publication,
                clear_switchover: false,
            },
            PostgresState::Offline => DesiredState {
                role: TargetRole::Idle(IdleReason::AwaitingLeader),
                publication,
                clear_switchover: false,
            },
            PostgresState::Replica { .. } => {
                if world.global.observed_primary.as_ref() == Some(&epoch.holder) {
                    DesiredState {
                        role: TargetRole::Follower(follow_goal(world, epoch.holder.clone())),
                        publication,
                        clear_switchover: false,
                    }
                } else {
                    DesiredState {
                        role: TargetRole::Idle(IdleReason::AwaitingLeader),
                        publication,
                        clear_switchover: false,
                    }
                }
            }
        };
    }

    match &world.global.lease {
        LeaseState::HeldByMe(epoch) => decide_as_lease_holder(world, self_id, epoch.clone()),
        LeaseState::HeldByPeer(epoch) => {
            let publication = PublicationGoal::PublishPrimary {
                primary: epoch.holder.clone(),
                epoch: epoch.clone(),
            };
            match &world.local.postgres {
                PostgresState::Primary { .. } => DesiredState {
                    role: TargetRole::Fenced(FenceReason::ForeignLeaderDetected),
                    publication,
                    clear_switchover: false,
                },
                PostgresState::Offline | PostgresState::Replica { .. } => DesiredState {
                    role: TargetRole::Follower(follow_goal(world, epoch.holder.clone())),
                    publication,
                    clear_switchover: false,
                },
            }
        }
        LeaseState::Unheld => decide_without_lease(world, self_id),
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
                    publication: PublicationGoal::PublishNoPrimary {
                        reason: NoPrimaryReason::DcsDegraded,
                        fence_cutoff: Some(cutoff),
                    },
                    clear_switchover: false,
                };
            }

            DesiredState {
                role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::DcsDegraded,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            }
        }
        PostgresState::Replica { upstream, .. } => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::ReplicaKeepFollowing(upstream.clone())),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        PostgresState::Offline => DesiredState {
            role: TargetRole::FailSafe(FailSafeGoal::WaitForQuorum),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::DcsDegraded,
                fence_cutoff: None,
            },
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

    match resolve_switchover(world, self_id, false) {
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
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Leader(epoch),
            publication,
            clear_switchover: true,
        },
    }
}

fn decide_without_lease(world: &WorldView, self_id: &MemberId) -> DesiredState {
    if let Some(leader) = world
        .global
        .observed_primary
        .clone()
        .filter(|leader| leader != self_id)
    {
        return DesiredState {
            role: TargetRole::Follower(follow_goal(world, leader)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        };
    }

    match resolve_switchover(world, self_id, true) {
        ResolvedSwitchover::Proceed(target) if target == *self_id => DesiredState {
            role: TargetRole::Candidate(Candidacy::TargetedSwitchover(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        ResolvedSwitchover::Proceed(target) => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingTarget(target)),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: false,
        },
        ResolvedSwitchover::Abandon => DesiredState {
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            publication: PublicationGoal::PublishNoPrimary {
                reason: NoPrimaryReason::LeaseOpen,
                fence_cutoff: None,
            },
            clear_switchover: true,
        },
        ResolvedSwitchover::NotRequested => match find_best_candidate(
            &world.global.peers,
            &world.global.self_peer,
            self_id,
        ) {
            Some(best) if best == *self_id => DesiredState {
                role: TargetRole::Candidate(candidacy_kind(world)),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            },
            Some(_) | None => DesiredState {
                role: TargetRole::Idle(IdleReason::AwaitingLeader),
                publication: PublicationGoal::PublishNoPrimary {
                    reason: NoPrimaryReason::LeaseOpen,
                    fence_cutoff: None,
                },
                clear_switchover: false,
            },
        },
    }
}

fn leader_publication(world: &WorldView, self_id: &MemberId, epoch: &LeaseEpoch) -> PublicationGoal {
    match &world.local.postgres {
        PostgresState::Primary { .. } => PublicationGoal::PublishPrimary {
            primary: self_id.clone(),
            epoch: epoch.clone(),
        },
        PostgresState::Offline | PostgresState::Replica { .. } => PublicationGoal::PublishNoPrimary {
            reason: NoPrimaryReason::Recovering,
            fence_cutoff: None,
        },
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
                PostgresState::Replica { .. } | PostgresState::Offline | PostgresState::Primary { .. } => {
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
            job: super::types::JobKind::PgRewind,
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
                world.local.publication.authority,
                super::types::AuthorityView::NoPrimary(NoPrimaryReason::DcsDegraded)
            ) {
                Candidacy::ResumeAfterOutage
            } else {
                Candidacy::Failover
            }
        }
    }
}

fn active_or_observed_epoch(world: &WorldView) -> Option<LeaseEpoch> {
    match &world.global.lease {
        LeaseState::HeldByMe(epoch) | LeaseState::HeldByPeer(epoch) => Some(epoch.clone()),
        LeaseState::Unheld => world.global.observed_lease.clone(),
    }
}

fn observed_foreign_lease(world: &WorldView, self_id: &MemberId) -> Option<LeaseEpoch> {
    world.global
        .observed_lease
        .clone()
        .filter(|epoch| &epoch.holder != self_id)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedSwitchover {
    NotRequested,
    Proceed(MemberId),
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
            .map_or(ResolvedSwitchover::Abandon, ResolvedSwitchover::Proceed),
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
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| switchover_target_is_valid(peer))
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_switchover_candidates(left_id, left_peer, right_id, right_peer)
        })
        .map(|(member_id, _)| member_id);

    if allow_self_target && switchover_target_is_valid(self_peer) {
        let self_candidate = Some(self_id.clone());
        return match (peer_candidate, self_candidate) {
            (Some(peer_id), Some(self_id)) => {
                if compare_self_to_peer(self_peer, &self_id, peers.get(&peer_id), &peer_id)
                    == Ordering::Greater
                {
                    Some(self_id)
                } else {
                    Some(peer_id)
                }
            }
            (Some(peer_id), None) => Some(peer_id),
            (None, Some(self_id)) => Some(self_id),
            (None, None) => None,
        };
    }

    peer_candidate
}

fn compare_self_to_peer(
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
    peer: Option<&PeerKnowledge>,
    peer_id: &MemberId,
) -> Ordering {
    match peer {
        Some(peer) => compare_switchover_candidates(self_id, self_peer, peer_id, peer),
        None => Ordering::Greater,
    }
}

fn switchover_target_is_valid(peer: &PeerKnowledge) -> bool {
    matches!(peer.api, ApiVisibility::Reachable)
        && matches!(peer.election, ElectionEligibility::PromoteEligible(_))
}

fn find_best_candidate(
    peers: &std::collections::BTreeMap<MemberId, PeerKnowledge>,
    self_peer: &PeerKnowledge,
    self_id: &MemberId,
) -> Option<MemberId> {
    let peer_candidate = peers
        .iter()
        .filter(|(_, peer)| classify_candidate(peer).is_some())
        .map(|(member_id, peer)| (member_id.clone(), peer))
        .max_by(|(left_id, left_peer), (right_id, right_peer)| {
            compare_candidates(left_id, left_peer, right_id, right_peer)
        })
        .map(|(member_id, _)| member_id);

    if classify_candidate(self_peer).is_none() {
        return peer_candidate;
    }

    match peer_candidate {
        Some(peer_id) => {
            if compare_self_to_peer(self_peer, self_id, peers.get(&peer_id), &peer_id)
                == Ordering::Greater
            {
                Some(self_id.clone())
            } else {
                Some(peer_id)
            }
        }
        None => Some(self_id.clone()),
    }
}

fn compare_switchover_candidates(
    left_id: &MemberId,
    left_peer: &PeerKnowledge,
    right_id: &MemberId,
    right_peer: &PeerKnowledge,
) -> Ordering {
    compare_candidate_rank(
        candidate_rank(&left_peer.election),
        left_id,
        candidate_rank(&right_peer.election),
        right_id,
    )
}

fn compare_candidates(
    left_id: &MemberId,
    left_peer: &PeerKnowledge,
    right_id: &MemberId,
    right_peer: &PeerKnowledge,
) -> Ordering {
    compare_candidate_rank(
        candidate_rank(&left_peer.election),
        left_id,
        candidate_rank(&right_peer.election),
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
        ElectionEligibility::PromoteEligible(position) => Some(CandidateRank::Promote(position.clone())),
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
        (Some(CandidateRank::Bootstrap), Some(CandidateRank::Bootstrap)) => {
            right_id.cmp(left_id)
        }
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}

fn classify_candidate(peer: &PeerKnowledge) -> Option<()> {
    match &peer.election {
        ElectionEligibility::BootstrapEligible | ElectionEligibility::PromoteEligible(_) => Some(()),
        ElectionEligibility::Ineligible(_) => None,
    }
}


===== src/ha/reconcile.rs =====
use super::types::{
    AuthorityView, DataDirState, DesiredState, FailSafeGoal, FenceReason, FollowGoal,
    IdleReason, LeaseState, LocalDataState, PostgresState, ProcessState, PublicationGoal,
    PublicationState, ReconcileAction, RecoveryPlan, TargetRole, WorldView,
};

pub(crate) fn reconcile(world: &WorldView, desired: &DesiredState) -> Vec<ReconcileAction> {
    let publication_actions = reconcile_publication(&world.local.publication, desired);
    let switchover_actions = reconcile_switchover(world, desired);
    let role_action = match &world.local.process {
        ProcessState::Running(_) => None,
        ProcessState::Idle | ProcessState::Failed(_) => reconcile_role(world, &desired.role),
    };

    publication_actions
        .into_iter()
        .chain(switchover_actions)
        .chain(role_action)
        .collect()
}

fn reconcile_publication(
    current: &PublicationState,
    desired: &DesiredState,
) -> Vec<ReconcileAction> {
    let publish_action = match (
        &current.authority,
        &current.fence_cutoff,
        &desired.publication,
    ) {
        (_, _, PublicationGoal::KeepCurrent) => None,
        (
            AuthorityView::Primary {
                member: current_member,
                epoch: current_epoch,
            },
            current_cutoff,
            PublicationGoal::PublishPrimary { primary, epoch },
        ) if current_member == primary && current_epoch == epoch && current_cutoff.is_none() => None,
        (
            AuthorityView::NoPrimary(current_reason),
            current_cutoff,
            PublicationGoal::PublishNoPrimary {
                reason,
                fence_cutoff,
            },
        ) if current_reason == reason && current_cutoff == fence_cutoff => None,
        (_, _, publication) => Some(ReconcileAction::Publish(publication.clone())),
    };

    publish_action.into_iter().collect()
}

fn reconcile_switchover(world: &WorldView, desired: &DesiredState) -> Vec<ReconcileAction> {
    match (&world.global.switchover, desired.clear_switchover) {
        (super::types::SwitchoverState::Requested(_), true) => {
            vec![ReconcileAction::ClearSwitchover]
        }
        (super::types::SwitchoverState::None, _) | (_, false) => Vec::new(),
    }
}

fn reconcile_role(world: &WorldView, target: &TargetRole) -> Option<ReconcileAction> {
    match target {
        TargetRole::Leader(_) => match (&world.local.data_dir, &world.local.postgres) {
            (DataDirState::Missing, _) => Some(ReconcileAction::InitDb),
            (DataDirState::Initialized(LocalDataState::BootstrapEmpty), _) => {
                Some(ReconcileAction::InitDb)
            }
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_start() => None,
            (DataDirState::Initialized(_), PostgresState::Offline) => {
                Some(ReconcileAction::StartPrimary)
            }
            (DataDirState::Initialized(_), PostgresState::Replica { .. }) => {
                Some(ReconcileAction::Promote)
            }
            (DataDirState::Initialized(_), PostgresState::Primary { .. }) => None,
        },
        TargetRole::Candidate(kind) => Some(ReconcileAction::AcquireLease(kind.clone())),
        TargetRole::Follower(goal) => reconcile_follow_role(world, goal),
        TargetRole::FailSafe(goal) => reconcile_failsafe_role(world, goal),
        TargetRole::DemotingForSwitchover(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
            }
            PostgresState::Offline => match &world.global.lease {
                LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
            },
        },
        TargetRole::Fenced(reason) => reconcile_fenced_role(world, reason),
        TargetRole::Idle(reason) => reconcile_idle_role(world, reason),
    }
}

fn reconcile_follow_role(world: &WorldView, goal: &FollowGoal) -> Option<ReconcileAction> {
    match goal.recovery {
        RecoveryPlan::None => None,
        RecoveryPlan::Basebackup => Some(ReconcileAction::BaseBackup(goal.leader.clone())),
        RecoveryPlan::Rewind => Some(ReconcileAction::PgRewind(goal.leader.clone())),
        RecoveryPlan::StartStreaming => {
            if world.local.observation.waiting_for_fresh_pg_after_start() {
                return None;
            }

            match &world.local.postgres {
                PostgresState::Offline => Some(ReconcileAction::StartReplica(goal.leader.clone())),
                PostgresState::Primary { .. } => {
                    Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast))
                }
                PostgresState::Replica { upstream, .. } => match upstream {
                    Some(current_upstream) if current_upstream == &goal.leader => None,
                    Some(_) | None => Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast)),
                },
            }
        }
    }
}

fn reconcile_failsafe_role(world: &WorldView, goal: &FailSafeGoal) -> Option<ReconcileAction> {
    match goal {
        FailSafeGoal::PrimaryMustStop(_) => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                Some(ReconcileAction::Demote(super::types::ShutdownMode::Immediate))
            }
            PostgresState::Offline => match &world.global.lease {
                LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
            },
        },
        FailSafeGoal::ReplicaKeepFollowing(_) => None,
        FailSafeGoal::WaitForQuorum => match &world.local.postgres {
            PostgresState::Primary { .. } => {
                Some(ReconcileAction::Demote(super::types::ShutdownMode::Immediate))
            }
            PostgresState::Replica { .. } => None,
            PostgresState::Offline => match &world.global.lease {
                LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
            },
        },
    }
}

fn reconcile_fenced_role(world: &WorldView, reason: &FenceReason) -> Option<ReconcileAction> {
    match reason {
        FenceReason::ForeignLeaderDetected | FenceReason::StorageStalled => {
            match &world.local.postgres {
                PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                    Some(ReconcileAction::Demote(super::types::ShutdownMode::Immediate))
                }
                PostgresState::Offline => match &world.global.lease {
                    LeaseState::HeldByMe(_) => Some(ReconcileAction::ReleaseLease),
                    LeaseState::HeldByPeer(_) | LeaseState::Unheld => None,
                },
            }
        }
    }
}

fn reconcile_idle_role(world: &WorldView, _reason: &IdleReason) -> Option<ReconcileAction> {
    match &world.local.postgres {
        PostgresState::Primary { .. } => Some(ReconcileAction::Demote(super::types::ShutdownMode::Fast)),
        PostgresState::Offline | PostgresState::Replica { .. } => None,
    }
}


===== src/debug_api/view.rs =====
use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    dcs::state::{DcsState, DcsTrust},
    debug_api::snapshot::{DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot},
    ha::state::HaState,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::state::{JobOutcome, ProcessState},
    state::{Versioned, WorkerStatus},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugVerbosePayload {
    pub(crate) meta: DebugMeta,
    pub(crate) config: ConfigSection,
    pub(crate) pginfo: PgInfoSection,
    pub(crate) dcs: DcsSection,
    pub(crate) process: ProcessSection,
    pub(crate) ha: HaSection,
    pub(crate) api: ApiSection,
    pub(crate) debug: DebugSection,
    pub(crate) changes: Vec<DebugChangeView>,
    pub(crate) timeline: Vec<DebugTimelineView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugMeta {
    pub(crate) schema_version: String,
    pub(crate) generated_at_ms: u64,
    pub(crate) channel_updated_at_ms: u64,
    pub(crate) channel_version: u64,
    pub(crate) app_lifecycle: String,
    pub(crate) sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) cluster_name: String,
    pub(crate) member_id: String,
    pub(crate) scope: String,
    pub(crate) debug_enabled: bool,
    pub(crate) tls_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgInfoSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) variant: String,
    pub(crate) worker: String,
    pub(crate) sql: String,
    pub(crate) readiness: String,
    pub(crate) timeline: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) trust: String,
    pub(crate) member_count: usize,
    pub(crate) leader: Option<String>,
    pub(crate) has_switchover_request: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) state: String,
    pub(crate) running_job_id: Option<String>,
    pub(crate) last_outcome: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) phase: String,
    pub(crate) tick: u64,
    pub(crate) decision: String,
    pub(crate) decision_detail: Option<String>,
    pub(crate) planned_actions: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiSection {
    pub(crate) endpoints: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugSection {
    pub(crate) history_changes: usize,
    pub(crate) history_timeline: usize,
    pub(crate) last_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugChangeView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) domain: String,
    pub(crate) previous_version: Option<u64>,
    pub(crate) current_version: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugTimelineView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) category: String,
    pub(crate) message: String,
}

pub(crate) fn build_verbose_payload(
    snapshot: &Versioned<SystemSnapshot>,
    since_sequence: Option<u64>,
) -> DebugVerbosePayload {
    let cutoff = since_sequence.unwrap_or(0);
    let filtered_changes = snapshot
        .value
        .changes
        .iter()
        .filter(|event| event.sequence > cutoff)
        .map(to_change_view)
        .collect::<Vec<_>>();
    let filtered_timeline = snapshot
        .value
        .timeline
        .iter()
        .filter(|entry| entry.sequence > cutoff)
        .map(to_timeline_view)
        .collect::<Vec<_>>();

    let cfg = &snapshot.value.config;
    let pg = &snapshot.value.pg;
    let dcs = &snapshot.value.dcs;
    let process = &snapshot.value.process;
    let ha = &snapshot.value.ha;

    DebugVerbosePayload {
        meta: DebugMeta {
            schema_version: "v1".to_string(),
            generated_at_ms: snapshot.value.generated_at.0,
            channel_updated_at_ms: snapshot.updated_at.0,
            channel_version: snapshot.version.0,
            app_lifecycle: format!("{:?}", snapshot.value.app),
            sequence: snapshot.value.sequence,
        },
        config: to_config_section(cfg),
        pginfo: to_pg_section(pg),
        dcs: to_dcs_section(dcs),
        process: to_process_section(process),
        ha: to_ha_section(ha),
        api: ApiSection {
            endpoints: vec![
                "/debug/snapshot".to_string(),
                "/debug/verbose".to_string(),
                "/debug/ui".to_string(),
                "/fallback/cluster".to_string(),
                "/switchover".to_string(),
                "/ha/state".to_string(),
                "/ha/switchover".to_string(),
            ],
        },
        debug: DebugSection {
            history_changes: snapshot.value.changes.len(),
            history_timeline: snapshot.value.timeline.len(),
            last_sequence: snapshot.value.sequence,
        },
        changes: filtered_changes,
        timeline: filtered_timeline,
    }
}

fn to_config_section(cfg: &Versioned<RuntimeConfig>) -> ConfigSection {
    ConfigSection {
        version: cfg.version.0,
        updated_at_ms: cfg.updated_at.0,
        cluster_name: cfg.value.cluster.name.clone(),
        member_id: cfg.value.cluster.member_id.clone(),
        scope: cfg.value.dcs.scope.clone(),
        debug_enabled: cfg.value.debug.enabled,
        tls_enabled: cfg.value.api.security.tls.mode != crate::config::ApiTlsMode::Disabled,
    }
}

fn to_pg_section(pg: &Versioned<PgInfoState>) -> PgInfoSection {
    match &pg.value {
        PgInfoState::Unknown { common } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Unknown".to_string(),
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "unknown worker={} sql={} readiness={}",
                worker_status_label(&common.worker),
                sql_label(&common.sql),
                readiness_label(&common.readiness)
            ),
        },
        PgInfoState::Primary {
            common,
            wal_lsn,
            slots,
        } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Primary".to_string(),
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "primary wal_lsn={} slots={} readiness={}",
                wal_lsn.0,
                slots.len(),
                readiness_label(&common.readiness)
            ),
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Replica".to_string(),
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "replica replay_lsn={} follow_lsn={} upstream={}",
                replay_lsn.0,
                follow_lsn
                    .map(|value| value.0)
                    .map_or_else(|| "none".to_string(), |value| value.to_string()),
                upstream
                    .as_ref()
                    .map(|value| value.member_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            ),
        },
    }
}

fn to_dcs_section(dcs: &Versioned<DcsState>) -> DcsSection {
    DcsSection {
        version: dcs.version.0,
        updated_at_ms: dcs.updated_at.0,
        worker: worker_status_label(&dcs.value.worker),
        trust: dcs_trust_label(&dcs.value.trust),
        member_count: dcs.value.cache.members.len(),
        leader: dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        has_switchover_request: dcs.value.cache.switchover.is_some(),
    }
}

fn to_process_section(process: &Versioned<ProcessState>) -> ProcessSection {
    match &process.value {
        ProcessState::Idle {
            worker,
            last_outcome,
        } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Idle".to_string(),
            running_job_id: None,
            last_outcome: last_outcome.as_ref().map(job_outcome_label),
        },
        ProcessState::Running { worker, active } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Running".to_string(),
            running_job_id: Some(active.id.0.clone()),
            last_outcome: None,
        },
    }
}

fn to_ha_section(ha: &Versioned<HaState>) -> HaSection {
    HaSection {
        version: ha.version.0,
        updated_at_ms: ha.updated_at.0,
        worker: worker_status_label(&ha.value.worker),
        phase: ha.value.role.label().to_string(),
        tick: ha.value.tick,
        decision: authority_label(&ha.value.publication.authority),
        decision_detail: Some(role_detail(&ha.value.role)),
        planned_actions: ha.value.planned_actions.len(),
    }
}

fn to_change_view(event: &DebugChangeEvent) -> DebugChangeView {
    DebugChangeView {
        sequence: event.sequence,
        at_ms: event.at.0,
        domain: debug_domain_label(&event.domain).to_string(),
        previous_version: event.previous_version.map(|value| value.0),
        current_version: event.current_version.map(|value| value.0),
        summary: event.summary.clone(),
    }
}

fn to_timeline_view(entry: &DebugTimelineEntry) -> DebugTimelineView {
    DebugTimelineView {
        sequence: entry.sequence,
        at_ms: entry.at.0,
        category: debug_domain_label(&entry.domain).to_string(),
        message: entry.message.clone(),
    }
}

fn worker_status_label(status: &WorkerStatus) -> String {
    match status {
        WorkerStatus::Starting => "Starting".to_string(),
        WorkerStatus::Running => "Running".to_string(),
        WorkerStatus::Stopping => "Stopping".to_string(),
        WorkerStatus::Stopped => "Stopped".to_string(),
        WorkerStatus::Faulted(error) => format!("Faulted({error})"),
    }
}

fn sql_label(status: &SqlStatus) -> String {
    match status {
        SqlStatus::Unknown => "Unknown".to_string(),
        SqlStatus::Healthy => "Healthy".to_string(),
        SqlStatus::Unreachable => "Unreachable".to_string(),
    }
}

fn readiness_label(readiness: &Readiness) -> String {
    match readiness {
        Readiness::Unknown => "Unknown".to_string(),
        Readiness::Ready => "Ready".to_string(),
        Readiness::NotReady => "NotReady".to_string(),
    }
}

fn dcs_trust_label(trust: &DcsTrust) -> String {
    match trust {
        DcsTrust::FullQuorum => "FullQuorum".to_string(),
        DcsTrust::FailSafe => "FailSafe".to_string(),
        DcsTrust::NotTrusted => "NotTrusted".to_string(),
    }
}

fn debug_domain_label(domain: &DebugDomain) -> &'static str {
    match domain {
        DebugDomain::App => "app",
        DebugDomain::Config => "config",
        DebugDomain::PgInfo => "pginfo",
        DebugDomain::Dcs => "dcs",
        DebugDomain::Process => "process",
        DebugDomain::Ha => "ha",
    }
}

fn authority_label(value: &crate::ha::types::AuthorityView) -> String {
    match value {
        crate::ha::types::AuthorityView::Primary { member, epoch } => {
            format!("primary:{}#{}", member.0, epoch.generation)
        }
        crate::ha::types::AuthorityView::NoPrimary(reason) => format!("no_primary:{reason:?}"),
        crate::ha::types::AuthorityView::Unknown => "unknown".to_string(),
    }
}

fn role_detail(value: &crate::ha::types::TargetRole) -> String {
    format!("{value:?}")
}

fn job_outcome_label(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { id, .. } => format!("Success({})", id.0),
        JobOutcome::Failure { id, error, .. } => format!("Failure({}: {:?})", id.0, error),
        JobOutcome::Timeout { id, .. } => format!("Timeout({})", id.0),
    }
}


===== src/cli/status.rs =====
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    time::Duration,
};

use reqwest::Url;
use serde::Serialize;
use tokio::task::JoinSet;

use crate::{
    api::{HaAuthorityResponse, HaClusterMemberResponse, HaStateResponse, TargetRoleResponse},
    cli::{
        args::StatusOptions,
        client::{CliApiClient, CliApiClientConfig, DebugVerboseResponse},
        config::OperatorContext,
        error::CliError,
        output,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClusterHealth {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiStatus {
    Ok,
    Down,
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugObservationStatus {
    Available,
    Disabled,
    AuthFailed,
    NotReady,
    TransportFailed,
    DecodeFailed,
    ApiStatusFailed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QueryOrigin {
    pub member_id: String,
    pub api_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterSwitchoverView {
    pub pending: bool,
    pub target_member_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterNodeDebugObservation {
    pub status: DebugObservationStatus,
    pub detail: Option<String>,
    pub payload: Option<DebugVerboseResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterNodeView {
    pub member_id: String,
    pub is_self: bool,
    pub sampled: bool,
    pub api_url: Option<String>,
    pub api_status: ApiStatus,
    pub role: String,
    pub trust: String,
    pub phase: String,
    pub leader: Option<String>,
    pub decision: Option<String>,
    pub pginfo: Option<String>,
    pub readiness: Option<String>,
    pub process: Option<String>,
    pub debug: Option<ClusterNodeDebugObservation>,
    pub observation_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterStatusView {
    pub cluster_name: String,
    pub scope: String,
    pub verbose: bool,
    pub queried_via: QueryOrigin,
    pub sampled_member_count: usize,
    pub discovered_member_count: usize,
    pub health: ClusterHealth,
    pub warnings: Vec<ClusterWarning>,
    pub switchover: Option<ClusterSwitchoverView>,
    pub nodes: Vec<ClusterNodeView>,
}

#[derive(Clone, Debug)]
pub(crate) struct SampledNodeState {
    pub(crate) state: HaStateResponse,
    pub(crate) debug: Option<ClusterNodeDebugObservation>,
}

#[derive(Clone, Debug)]
pub(crate) struct PeerObservation {
    pub(crate) member_id: String,
    pub(crate) sampled: Result<SampledNodeState, String>,
}

#[derive(Clone, Debug)]
pub(crate) struct SampledClusterSnapshot {
    pub(crate) seed_state: HaStateResponse,
    pub(crate) discovered_members: Vec<HaClusterMemberResponse>,
    pub(crate) queried_via: QueryOrigin,
    pub(crate) observations: BTreeMap<String, PeerObservation>,
    pub(crate) warnings: Vec<ClusterWarning>,
}

impl SampledClusterSnapshot {
    pub(crate) fn sampled_member_count(&self) -> usize {
        self.observations
            .values()
            .filter(|value| value.sampled.is_ok())
            .count()
    }

    pub(crate) fn discovered_member_count(&self) -> usize {
        self.discovered_members.len()
    }
}

pub(crate) async fn run_status(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<String, CliError> {
    if options.watch {
        run_watch(context, options).await
    } else {
        let view = build_cluster_status_view(context, options).await?;
        output::render_status_view(&view, options.json)
    }
}

pub(crate) async fn build_cluster_status_view(
    context: &OperatorContext,
    options: StatusOptions,
) -> Result<ClusterStatusView, CliError> {
    let snapshot = build_sampled_cluster_snapshot(context, options.verbose).await?;
    Ok(assemble_cluster_view(&snapshot, options.verbose))
}

pub(crate) async fn build_sampled_cluster_snapshot(
    context: &OperatorContext,
    verbose: bool,
) -> Result<SampledClusterSnapshot, CliError> {
    let seed_client = CliApiClient::from_config(context.api_client.clone())?;
    let seed_state = seed_client.get_ha_state().await?;
    let seed_debug = fetch_debug_observation(&seed_client, verbose).await;
    let discovered_members = seed_state.members.clone();
    let queried_via = QueryOrigin {
        member_id: seed_state.self_member_id.clone(),
        api_url: seed_client.base_url().to_string(),
    };

    let peer_observations =
        sample_peer_states(&context.api_client, &seed_state, seed_debug, verbose).await;
    let warnings = collect_warnings(&seed_state, &discovered_members, &peer_observations);

    Ok(SampledClusterSnapshot {
        seed_state,
        discovered_members,
        queried_via,
        observations: peer_observations,
        warnings,
    })
}

async fn run_watch(context: &OperatorContext, options: StatusOptions) -> Result<String, CliError> {
    let mut stdout = std::io::stdout();
    let interval = Duration::from_secs(2);

    loop {
        let view = build_cluster_status_view(context, options).await?;
        let rendered = output::render_status_view(&view, options.json)?;
        if options.json {
            writeln!(stdout, "{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        } else {
            writeln!(stdout, "\x1B[2J\x1B[H{rendered}")
                .map_err(|err| CliError::Output(format!("watch write failed: {err}")))?;
        }
        stdout
            .flush()
            .map_err(|err| CliError::Output(format!("watch flush failed: {err}")))?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(String::new()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

async fn sample_peer_states(
    base_config: &CliApiClientConfig,
    seed_state: &HaStateResponse,
    seed_debug: Option<ClusterNodeDebugObservation>,
    verbose: bool,
) -> BTreeMap<String, PeerObservation> {
    let seed_observation = PeerObservation {
        member_id: seed_state.self_member_id.clone(),
        sampled: Ok(SampledNodeState {
            state: seed_state.clone(),
            debug: seed_debug,
        }),
    };

    let mut observations = BTreeMap::from([(seed_state.self_member_id.clone(), seed_observation)]);
    let mut join_set = JoinSet::new();

    for member in &seed_state.members {
        if member.member_id == seed_state.self_member_id {
            continue;
        }

        let Some(api_url) = member.api_url.as_deref() else {
            observations.insert(
                member.member_id.clone(),
                PeerObservation {
                    member_id: member.member_id.clone(),
                    sampled: Err("missing advertised api_url".to_string()),
                },
            );
            continue;
        };

        let parsed_url = match Url::parse(api_url) {
            Ok(value) => value,
            Err(err) => {
                observations.insert(
                    member.member_id.clone(),
                    PeerObservation {
                        member_id: member.member_id.clone(),
                        sampled: Err(format!("invalid advertised api_url `{api_url}`: {err}")),
                    },
                );
                continue;
            }
        };

        let config = base_config.with_base_url(parsed_url);
        let member_id = member.member_id.clone();
        join_set.spawn(async move {
            match CliApiClient::from_config(config) {
                Ok(client) => match client.get_ha_state().await {
                    Ok(state) => {
                        let debug_observation = fetch_debug_observation(&client, verbose).await;
                        PeerObservation {
                            member_id: member_id.clone(),
                            sampled: Ok(SampledNodeState {
                                state,
                                debug: debug_observation,
                            }),
                        }
                    }
                    Err(err) => PeerObservation {
                        member_id: member_id.clone(),
                        sampled: Err(err.to_string()),
                    },
                },
                Err(err) => PeerObservation {
                    member_id: member_id.clone(),
                    sampled: Err(err.to_string()),
                },
            }
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(observation) => {
                observations.insert(observation.member_id.clone(), observation);
            }
            Err(err) => {
                let member_id = format!("task-join-error-{}", observations.len());
                observations.insert(
                    member_id.clone(),
                    PeerObservation {
                        member_id,
                        sampled: Err(format!("peer sampling task failed: {err}")),
                    },
                );
            }
        }
    }

    observations
}

async fn fetch_debug_observation(
    client: &CliApiClient,
    verbose: bool,
) -> Option<ClusterNodeDebugObservation> {
    if !verbose {
        return None;
    }

    match client.get_debug_verbose().await {
        Ok(value) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::Available,
            detail: None,
            payload: Some(value),
        }),
        Err(CliError::ApiStatus { status, body }) if status == 404 => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::Disabled,
                detail: summarize_api_failure(status, body.as_str()),
                payload: None,
            })
        }
        Err(CliError::ApiStatus { status, body }) if status == 401 || status == 403 => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::AuthFailed,
                detail: summarize_api_failure(status, body.as_str()),
                payload: None,
            })
        }
        Err(CliError::ApiStatus { status, body }) if status == 503 => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::NotReady,
                detail: summarize_api_failure(status, body.as_str()),
                payload: None,
            })
        }
        Err(CliError::ApiStatus { status, body }) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::ApiStatusFailed,
            detail: summarize_api_failure(status, body.as_str()),
            payload: None,
        }),
        Err(CliError::Transport(message) | CliError::RequestBuild(message)) => {
            Some(ClusterNodeDebugObservation {
                status: DebugObservationStatus::TransportFailed,
                detail: Some(message),
                payload: None,
            })
        }
        Err(CliError::Decode(message)) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::DecodeFailed,
            detail: Some(message),
            payload: None,
        }),
        Err(
            CliError::Config(message) | CliError::Resolution(message) | CliError::Output(message),
        ) => Some(ClusterNodeDebugObservation {
            status: DebugObservationStatus::ApiStatusFailed,
            detail: Some(message),
            payload: None,
        }),
    }
}

fn summarize_api_failure(status: u16, body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Some(format!("http {status}"));
    }

    let first_line = match trimmed.lines().next() {
        Some(value) => value,
        None => trimmed,
    };
    let clipped = if first_line.chars().count() > 120 {
        let shortened = first_line.chars().take(120).collect::<String>();
        format!("{shortened}...")
    } else {
        first_line.to_string()
    };
    Some(format!("http {status}: {clipped}"))
}

fn assemble_cluster_view(snapshot: &SampledClusterSnapshot, verbose: bool) -> ClusterStatusView {
    let mut nodes = snapshot
        .discovered_members
        .iter()
        .map(|member| {
            build_node_row(
                member,
                snapshot.queried_via.member_id.as_str(),
                snapshot.observations.get(&member.member_id),
            )
        })
        .collect::<Vec<_>>();
    nodes.sort_by(node_sort_key);

    ClusterStatusView {
        cluster_name: snapshot.seed_state.cluster_name.clone(),
        scope: snapshot.seed_state.scope.clone(),
        verbose,
        queried_via: snapshot.queried_via.clone(),
        sampled_member_count: snapshot.sampled_member_count(),
        discovered_member_count: snapshot.discovered_member_count(),
        health: if snapshot.warnings.is_empty() {
            ClusterHealth::Healthy
        } else {
            ClusterHealth::Degraded
        },
        warnings: snapshot.warnings.clone(),
        switchover: snapshot
            .seed_state
            .switchover_pending
            .then_some(ClusterSwitchoverView {
                pending: true,
                target_member_id: snapshot.seed_state.switchover_to.clone(),
            }),
        nodes,
    }
}

fn collect_warnings(
    seed_state: &HaStateResponse,
    discovered_members: &[HaClusterMemberResponse],
    observations: &BTreeMap<String, PeerObservation>,
) -> Vec<ClusterWarning> {
    let mut warnings = Vec::new();
    let mut sampled_leaders = BTreeSet::new();
    let mut sampled_primary_members = BTreeSet::new();
    let seed_members = discovered_members
        .iter()
        .map(|member| member.member_id.clone())
        .collect::<BTreeSet<_>>();

    for member in discovered_members {
        let observation = observations.get(&member.member_id);
        match observation {
            Some(PeerObservation {
                sampled: Ok(sampled),
                ..
            }) => {
                sampled_leaders.insert(
                    authority_primary_member(&sampled.state)
                        .or_else(|| sampled.state.leader.clone())
                        .unwrap_or_else(|| "<none>".to_string()),
                );
                if observed_role(member, &sampled.state) == "primary" {
                    sampled_primary_members.insert(sampled.state.self_member_id.clone());
                }
                if sampled.state.dcs_trust != crate::api::DcsTrustResponse::FullQuorum {
                    warnings.push(ClusterWarning {
                        code: "degraded_trust".to_string(),
                        message: format!(
                            "node {} reports trust {}",
                            sampled.state.self_member_id, sampled.state.dcs_trust
                        ),
                    });
                }
                let sampled_members = sampled
                    .state
                    .members
                    .iter()
                    .map(|value| value.member_id.clone())
                    .collect::<BTreeSet<_>>();
                if sampled_members != seed_members {
                    warnings.push(ClusterWarning {
                        code: "membership_mismatch".to_string(),
                        message: format!(
                            "node {} reports a different member set than queried_via {}",
                            sampled.state.self_member_id, seed_state.self_member_id
                        ),
                    });
                }
            }
            Some(PeerObservation {
                sampled: Err(message),
                ..
            }) => warnings.push(ClusterWarning {
                code: if member.api_url.is_some() {
                    "unreachable_node".to_string()
                } else {
                    "missing_api_url".to_string()
                },
                message: format!("node {} could not be sampled: {message}", member.member_id),
            }),
            None => warnings.push(ClusterWarning {
                code: "missing_observation".to_string(),
                message: format!("node {} was not sampled", member.member_id),
            }),
        }
    }

    if sampled_leaders.len() > 1 {
        warnings.push(ClusterWarning {
            code: "leader_mismatch".to_string(),
            message: format!(
                "sampled nodes disagree on leader: {}",
                sampled_leaders.into_iter().collect::<Vec<_>>().join(", ")
            ),
        });
    }

    if sampled_primary_members.len() > 1 {
        warnings.push(ClusterWarning {
            code: "multi_primary".to_string(),
            message: format!(
                "multiple sampled primaries: {}",
                sampled_primary_members
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }

    let sampled_member_count = observations
        .values()
        .filter(|value| value.sampled.is_ok())
        .count();
    if sampled_member_count < discovered_members.len() {
        warnings.push(ClusterWarning {
            code: "insufficient_sampling".to_string(),
            message: format!(
                "sampled {sampled_member_count}/{} discovered members",
                discovered_members.len()
            ),
        });
    }

    warnings
}

fn build_node_row(
    member: &HaClusterMemberResponse,
    queried_member_id: &str,
    observation: Option<&PeerObservation>,
) -> ClusterNodeView {
    match observation {
        Some(PeerObservation {
            sampled: Ok(sampled),
            ..
        }) => {
            let debug_payload = sampled
                .debug
                .as_ref()
                .and_then(|observation| observation.payload.as_ref());
            ClusterNodeView {
                member_id: member.member_id.clone(),
                is_self: member.member_id == queried_member_id,
                sampled: true,
                api_url: member.api_url.clone(),
                api_status: ApiStatus::Ok,
                role: observed_role(member, &sampled.state).to_string(),
                trust: sampled.state.dcs_trust.to_string(),
                phase: render_role_text(&sampled.state.ha_role),
                leader: authority_primary_member(&sampled.state).or_else(|| sampled.state.leader.clone()),
                decision: Some(render_authority_text(&sampled.state.authority)),
                pginfo: debug_payload.map(|value| value.pginfo.summary.clone()),
                readiness: debug_payload.map(|value| value.pginfo.readiness.to_ascii_lowercase()),
                process: debug_payload.map(|value| value.process.state.to_ascii_lowercase()),
                debug: sampled.debug.clone(),
                observation_error: None,
            }
        }
        Some(PeerObservation {
            sampled: Err(message),
            ..
        }) => ClusterNodeView {
            member_id: member.member_id.clone(),
            is_self: member.member_id == queried_member_id,
            sampled: false,
            api_url: member.api_url.clone(),
            api_status: if member.api_url.is_some() {
                ApiStatus::Down
            } else {
                ApiStatus::Missing
            },
            role: "unknown".to_string(),
            trust: "unknown".to_string(),
            phase: "unknown".to_string(),
            leader: None,
            decision: None,
            pginfo: None,
            readiness: None,
            process: None,
            debug: None,
            observation_error: Some(message.clone()),
        },
        None => ClusterNodeView {
            member_id: member.member_id.clone(),
            is_self: member.member_id == queried_member_id,
            sampled: false,
            api_url: member.api_url.clone(),
            api_status: ApiStatus::Missing,
            role: "unknown".to_string(),
            trust: "unknown".to_string(),
            phase: "unknown".to_string(),
            leader: None,
            decision: None,
            pginfo: None,
            readiness: None,
            process: None,
            debug: None,
            observation_error: Some("no observation recorded".to_string()),
        },
    }
}

fn node_sort_key(left: &ClusterNodeView, right: &ClusterNodeView) -> std::cmp::Ordering {
    (
        !left.is_self,
        role_rank(left.role.as_str()),
        left.member_id.as_str(),
    )
        .cmp(&(
            !right.is_self,
            role_rank(right.role.as_str()),
            right.member_id.as_str(),
        ))
}

fn role_rank(role: &str) -> u8 {
    match role {
        "primary" => 0,
        "replica" => 1,
        _ => 2,
    }
}

pub(crate) fn observed_role(member: &HaClusterMemberResponse, state: &HaStateResponse) -> &'static str {
    if !matches!(state.dcs_trust, crate::api::DcsTrustResponse::FullQuorum) {
        return "unknown";
    }

    if authority_primary_member(state).as_deref() == Some(member.member_id.as_str()) {
        return "primary";
    }

    match member.role {
        crate::api::MemberRoleResponse::Replica => "replica",
        crate::api::MemberRoleResponse::Primary | crate::api::MemberRoleResponse::Unknown => "unknown",
    }
}

fn render_authority_text(value: &HaAuthorityResponse) -> String {
    match value {
        HaAuthorityResponse::Primary { member_id, epoch } => {
            format!("primary(member_id={member_id}, generation={})", epoch.generation)
        }
        HaAuthorityResponse::NoPrimary { reason } => format!("no_primary(reason={reason:?})"),
        HaAuthorityResponse::Unknown => "unknown".to_string(),
    }
}

fn render_role_text(value: &TargetRoleResponse) -> String {
    match value {
        TargetRoleResponse::Leader { .. } => "leader".to_string(),
        TargetRoleResponse::Candidate { candidacy } => format!("candidate({candidacy:?})"),
        TargetRoleResponse::Follower { goal } => format!("follower({:?})", goal.recovery),
        TargetRoleResponse::FailSafe { goal } => format!("fail_safe({goal:?})"),
        TargetRoleResponse::DemotingForSwitchover { member_id } => {
            format!("demoting_for_switchover({member_id})")
        }
        TargetRoleResponse::Fenced { reason } => format!("fenced({reason:?})"),
        TargetRoleResponse::Idle { reason } => format!("idle({reason:?})"),
    }
}

fn authority_primary_member(state: &HaStateResponse) -> Option<String> {
    match &state.authority {
        HaAuthorityResponse::Primary { member_id, .. } => Some(member_id.clone()),
        HaAuthorityResponse::NoPrimary { .. } | HaAuthorityResponse::Unknown => None,
    }
}

#[cfg(all(test, any()))]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        api::{
            DcsTrustResponse, HaClusterMemberResponse, HaDecisionResponse, HaPhaseResponse,
            HaStateResponse, MemberRoleResponse, ReadinessResponse, SqlStatusResponse,
        },
        cli::{
            client::DebugVerboseResponse,
            status::{
                assemble_cluster_view, ApiStatus, ClusterNodeDebugObservation,
                DebugObservationStatus, QueryOrigin, SampledClusterSnapshot,
            },
        },
        debug_api::view::{
            ApiSection, ConfigSection, DcsSection, DebugChangeView, DebugMeta, DebugSection,
            DebugTimelineView, HaSection, PgInfoSection, ProcessSection,
        },
    };

    fn sample_member(member_id: &str, api_url: Option<&str>) -> HaClusterMemberResponse {
        HaClusterMemberResponse {
            member_id: member_id.to_string(),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: api_url.map(ToString::to_string),
            role: MemberRoleResponse::Unknown,
            sql: SqlStatusResponse::Healthy,
            readiness: ReadinessResponse::Ready,
            timeline: Some(7),
            write_lsn: None,
            replay_lsn: Some(5),
            updated_at_ms: 1,
            pg_version: 1,
        }
    }

    fn sample_state(
        self_member_id: &str,
        phase: HaPhaseResponse,
        trust: DcsTrustResponse,
        leader: Option<&str>,
        members: Vec<HaClusterMemberResponse>,
    ) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            self_member_id: self_member_id.to_string(),
            leader: leader.map(ToString::to_string),
            switchover_pending: false,
            switchover_to: None,
            member_count: members.len(),
            members,
            dcs_trust: trust,
            ha_phase: phase,
            ha_tick: 1,
            ha_decision: HaDecisionResponse::NoChange,
            snapshot_sequence: 10,
        }
    }

    fn sample_snapshot(
        seed_state: HaStateResponse,
        discovered_members: Vec<HaClusterMemberResponse>,
        observations: BTreeMap<String, super::PeerObservation>,
    ) -> SampledClusterSnapshot {
        let warnings = super::collect_warnings(&seed_state, &discovered_members, &observations);
        SampledClusterSnapshot {
            seed_state,
            discovered_members,
            queried_via: QueryOrigin {
                member_id: "node-a".to_string(),
                api_url: "http://node-a:8080".to_string(),
            },
            observations,
            warnings,
        }
    }

    fn sample_debug_payload(member_id: &str) -> DebugVerboseResponse {
        DebugVerboseResponse {
            meta: DebugMeta {
                schema_version: "v1".to_string(),
                generated_at_ms: 1,
                channel_updated_at_ms: 1,
                channel_version: 1,
                app_lifecycle: "Running".to_string(),
                sequence: 42,
            },
            config: ConfigSection {
                version: 1,
                updated_at_ms: 1,
                cluster_name: "cluster-a".to_string(),
                member_id: member_id.to_string(),
                scope: "scope-a".to_string(),
                debug_enabled: true,
                tls_enabled: false,
            },
            pginfo: PgInfoSection {
                version: 1,
                updated_at_ms: 1,
                variant: "Primary".to_string(),
                worker: "Running".to_string(),
                sql: "Healthy".to_string(),
                readiness: "Ready".to_string(),
                timeline: Some(7),
                summary: "primary wal_lsn=7 readiness=Ready".to_string(),
            },
            dcs: DcsSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                trust: "FullQuorum".to_string(),
                member_count: 1,
                leader: Some("node-a".to_string()),
                has_switchover_request: false,
            },
            process: ProcessSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                state: "Idle".to_string(),
                running_job_id: None,
                last_outcome: Some("Success(job-1)".to_string()),
            },
            ha: HaSection {
                version: 1,
                updated_at_ms: 1,
                worker: "Running".to_string(),
                phase: "Primary".to_string(),
                tick: 1,
                decision: "NoChange".to_string(),
                decision_detail: Some("steady".to_string()),
                planned_actions: 0,
            },
            api: ApiSection {
                endpoints: vec!["/debug/verbose".to_string()],
            },
            debug: DebugSection {
                history_changes: 1,
                history_timeline: 1,
                last_sequence: 42,
            },
            changes: vec![DebugChangeView {
                sequence: 41,
                at_ms: 1,
                domain: "ha".to_string(),
                previous_version: Some(1),
                current_version: Some(2),
                summary: "decision updated".to_string(),
            }],
            timeline: vec![DebugTimelineView {
                sequence: 42,
                at_ms: 1,
                category: "ha".to_string(),
                message: "primary steady".to_string(),
            }],
        }
    }

    #[test]
    fn assemble_cluster_view_marks_missing_api_targets_as_degraded() {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", None),
        ];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                super::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(super::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                super::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Err("missing advertised api_url".to_string()),
                },
            ),
        ]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.health, super::ClusterHealth::Degraded);
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "missing_api_url"));
        assert!(view
            .nodes
            .iter()
            .any(|node| node.api_status == ApiStatus::Missing));
    }

    #[test]
    fn assemble_cluster_view_marks_multi_primary_as_degraded() {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", Some("http://node-b:8080")),
        ];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let other_state = sample_state(
            "node-b",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-b"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                super::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(super::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                super::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Ok(super::SampledNodeState {
                        state: other_state,
                        debug: None,
                    }),
                },
            ),
        ]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.health, super::ClusterHealth::Degraded);
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "multi_primary"));
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "leader_mismatch"));
    }

    #[test]
    fn assemble_cluster_view_marks_degraded_trust_as_degraded() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FailSafe,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.health, super::ClusterHealth::Degraded);
        assert!(view
            .warnings
            .iter()
            .any(|warning| warning.code == "degraded_trust"));
    }

    #[test]
    fn assemble_cluster_view_preserves_verbose_mode_without_debug_payload() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, true);

        assert!(view.verbose);
        assert_eq!(view.nodes[0].pginfo, None);
    }

    #[test]
    fn assemble_cluster_view_preserves_debug_observation_reasons() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: Some(ClusterNodeDebugObservation {
                        status: DebugObservationStatus::AuthFailed,
                        detail: Some("http 401: missing token".to_string()),
                        payload: None,
                    }),
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, true);

        assert!(view.verbose);
        assert_eq!(view.nodes[0].pginfo, None);
        assert_eq!(
            view.nodes[0].debug.as_ref().map(|value| &value.status),
            Some(&DebugObservationStatus::AuthFailed)
        );
    }

    #[test]
    fn assemble_cluster_view_includes_debug_payload_summary_when_available() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            HaPhaseResponse::Primary,
            DcsTrustResponse::FullQuorum,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: seed_state.clone(),
                    debug: Some(ClusterNodeDebugObservation {
                        status: DebugObservationStatus::Available,
                        detail: None,
                        payload: Some(sample_debug_payload("node-a")),
                    }),
                }),
            },
        )]);

        let snapshot = sample_snapshot(seed_state, members, observations);
        let view = assemble_cluster_view(&snapshot, true);

        assert_eq!(
            view.nodes[0].pginfo.as_deref(),
            Some("primary wal_lsn=7 readiness=Ready")
        );
        assert_eq!(view.nodes[0].process.as_deref(), Some("idle"));
        assert_eq!(
            view.nodes[0].debug.as_ref().map(|value| &value.status),
            Some(&DebugObservationStatus::Available)
        );
    }

    #[test]
    fn assemble_cluster_view_prefers_member_role_over_transitional_phase() {
        let mut members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        members[0].role = MemberRoleResponse::Replica;
        let transitional_state = sample_state(
            "node-a",
            HaPhaseResponse::WaitingDcsTrusted,
            DcsTrustResponse::FullQuorum,
            Some("node-b"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: transitional_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(transitional_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.nodes[0].role, "replica");
    }

    #[test]
    fn assemble_cluster_view_does_not_surface_failsafe_member_as_primary() {
        let mut members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        members[0].role = MemberRoleResponse::Primary;
        let failsafe_state = sample_state(
            "node-a",
            HaPhaseResponse::FailSafe,
            DcsTrustResponse::FailSafe,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: failsafe_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(failsafe_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.nodes[0].role, "unknown");
    }

    #[test]
    fn assemble_cluster_view_does_not_surface_waiting_member_without_full_quorum() {
        let mut members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        members[0].role = MemberRoleResponse::Primary;
        let waiting_state = sample_state(
            "node-a",
            HaPhaseResponse::WaitingDcsTrusted,
            DcsTrustResponse::FailSafe,
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            super::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(super::SampledNodeState {
                    state: waiting_state.clone(),
                    debug: None,
                }),
            },
        )]);

        let snapshot = sample_snapshot(waiting_state, members, observations);
        let view = assemble_cluster_view(&snapshot, false);

        assert_eq!(view.nodes[0].role, "unknown");
    }
}
