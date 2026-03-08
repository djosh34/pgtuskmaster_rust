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

docs/src/how-to/run-tests.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/how-to/bootstrap-cluster.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/handle-primary-failure.md
docs/src/how-to/perform-switchover.md
docs/src/reference/http-api.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/pgtuskmasterctl-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md


# current docs summary context

===== docs/src/SUMMARY.md =====
# Summary

# Tutorials
- [Tutorials]()
    - [First HA Cluster](tutorial/first-ha-cluster.md)
    - [Observing a Failover Event](tutorial/observing-failover.md)

# How-To

- [How-To]()
    - [Bootstrap a New Cluster from Zero State](how-to/bootstrap-cluster.md)
    - [Check Cluster Health](how-to/check-cluster-health.md)
    - [Handle Primary Failure](how-to/handle-primary-failure.md)
    - [Perform a Planned Switchover](how-to/perform-switchover.md)

# Explanation

- [Explanation]()
    - [Architecture](explanation/architecture.md)
    - [Failure Modes and Recovery Behavior](explanation/failure-modes.md)

# Reference

- [Reference]()
    - [HTTP API](reference/http-api.md)
    - [pgtuskmaster CLI](reference/pgtuskmaster-cli.md)
    - [pgtuskmasterctl CLI](reference/pgtuskmasterctl-cli.md)
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
tokio = { version = "1.44.1", features = ["sync", "rt", "rt-multi-thread", "macros", "time", "process", "net", "io-util", "fs"] }
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
src/bin/pgtuskmaster.rs
src/bin/pgtuskmasterctl.rs
src/cli/args.rs
src/cli/client.rs
src/cli/error.rs
src/cli/mod.rs
src/cli/output.rs
src/config/defaults.rs
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
tests/policy_e2e_api_only.rs


# docker and docs support file listing

docker/Dockerfile.dev
docker/Dockerfile.prod
docker/compose/docker-compose.cluster.yml
docker/compose/docker-compose.single.yml
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
docs/draft/docs/src/explanation/introduction.md
docs/draft/docs/src/how-to/bootstrap-cluster.md
docs/draft/docs/src/how-to/bootstrap-cluster.revised.md
docs/draft/docs/src/how-to/check-cluster-health.md
docs/draft/docs/src/how-to/check-cluster-health.revised.md
docs/draft/docs/src/how-to/configure-tls-security.md
docs/draft/docs/src/how-to/configure-tls.md
docs/draft/docs/src/how-to/debug-cluster-issues.md
docs/draft/docs/src/how-to/handle-primary-failure.md
docs/draft/docs/src/how-to/handle-primary-failure.revised.md
docs/draft/docs/src/how-to/perform-switchover.md
docs/draft/docs/src/how-to/perform-switchover.revised.md
docs/draft/docs/src/how-to/run-tests.md
docs/draft/docs/src/reference/cli-commands.md
docs/draft/docs/src/reference/cli-commands.revised.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.revised.md
docs/draft/docs/src/reference/cli.md
docs/draft/docs/src/reference/cli.revised.md
docs/draft/docs/src/reference/http-api.md
docs/draft/docs/src/reference/http-api.revised.md
docs/draft/docs/src/reference/pgtuskmaster-cli.md
docs/draft/docs/src/reference/pgtuskmaster-cli.revised.md
docs/draft/docs/src/reference/pgtuskmasterctl-cli.md
docs/draft/docs/src/reference/pgtuskmasterctl-cli.revised.md
docs/draft/docs/src/reference/runtime-configuration.md
docs/draft/docs/src/reference/runtime-configuration.revised.md
docs/draft/docs/src/tutorial/first-ha-cluster.final.md
docs/draft/docs/src/tutorial/first-ha-cluster.md
docs/draft/docs/src/tutorial/first-ha-cluster.revised.md
docs/draft/docs/src/tutorial/observing-failover.md
docs/draft/docs/src/tutorial/observing-failover.revised.md
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/explanation/failure-modes.md
docs/src/how-to/bootstrap-cluster.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/handle-primary-failure.md
docs/src/how-to/perform-switchover.md
docs/src/reference/http-api.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/pgtuskmasterctl-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/src/tutorial/observing-failover.md
docs/tmp/docs/src/explanation/architecture.prompt.md
docs/tmp/docs/src/explanation/failure-modes.prompt.md
docs/tmp/docs/src/explanation/introduction.prompt.md
docs/tmp/docs/src/how-to/bootstrap-cluster.prompt.md
docs/tmp/docs/src/how-to/check-cluster-health.prompt.md
docs/tmp/docs/src/how-to/configure-tls-security.prompt.md
docs/tmp/docs/src/how-to/configure-tls.prompt.md
docs/tmp/docs/src/how-to/debug-cluster-issues.prompt.md
docs/tmp/docs/src/how-to/handle-primary-failure.prompt.md
docs/tmp/docs/src/how-to/perform-switchover.prompt.md
docs/tmp/docs/src/how-to/run-tests.prompt.md
docs/tmp/docs/src/reference/cli-commands.prompt.md
docs/tmp/docs/src/reference/cli-pgtuskmasterctl.prompt.md
docs/tmp/docs/src/reference/cli.prompt.md
docs/tmp/docs/src/reference/http-api.prompt.md
docs/tmp/docs/src/reference/pgtuskmaster-cli.prompt.md
docs/tmp/docs/src/reference/pgtuskmasterctl-cli.prompt.md
docs/tmp/docs/src/reference/runtime-configuration.prompt.md
docs/tmp/docs/src/tutorial/first-ha-cluster.prompt.md
docs/tmp/docs/src/tutorial/observing-failover.prompt.md
docs/tmp/k2-batch/20260308-architecture.prepare.out
docs/tmp/k2-batch/20260308-batch2-lane1.choose.md
docs/tmp/k2-batch/20260308-batch2-lane2.choose.md
docs/tmp/k2-batch/20260308-batch2-lane3.choose.md
docs/tmp/k2-batch/20260308-batch2-lane4.choose.md
docs/tmp/k2-batch/20260308-batch2-lane5.choose.md
docs/tmp/k2-batch/20260308-batch2-runtime.prepare.out
docs/tmp/k2-batch/20260308-batch3-reroll/lane2.choose.md
docs/tmp/k2-batch/20260308-batch3-reroll/lane3.choose.md
docs/tmp/k2-batch/20260308-batch3-reroll/lane4.choose.md
docs/tmp/k2-batch/20260308-batch3-reroll/lane5.choose.md
docs/tmp/k2-batch/20260308-batch3/lane1.choose.md
docs/tmp/k2-batch/20260308-batch3/lane2.choose.md
docs/tmp/k2-batch/20260308-batch3/lane3.choose.md
docs/tmp/k2-batch/20260308-batch3/lane4.choose.md
docs/tmp/k2-batch/20260308-batch3/lane5.choose.md
docs/tmp/k2-batch/20260308-lane1.choose.md
docs/tmp/k2-batch/20260308-lane2.choose.md
docs/tmp/k2-batch/20260308-lane3.choose.md
docs/tmp/k2-batch/20260308-lane4.choose.md
docs/tmp/k2-batch/20260308-lane5.choose.md
docs/tmp/k2-batch/20260308-reroll-lane1.choose.md
docs/tmp/k2-batch/20260308-reroll-lane3.choose.md
docs/tmp/k2-batch/20260308-reroll-lane4.choose.md
docs/tmp/k2-batch/20260308-reroll-lane5.choose.md
docs/tmp/k2-batch/20260308-runtime.prepare.out
docs/tmp/k2-batch/lane1.choose.md
docs/tmp/k2-batch/lane1.prepare.out
docs/tmp/k2-batch/lane2.choose.md
docs/tmp/k2-batch/lane2.prepare.out
docs/tmp/k2-batch/lane3.choose.md
docs/tmp/k2-batch/lane3.prepare.out
docs/tmp/k2-batch/lane4.choose.md
docs/tmp/k2-batch/lane4.prepare.out
docs/tmp/k2-batch/lane5.choose.md
docs/tmp/k2-batch/lane5.prepare.out
docs/tmp/verbose_extra_context/architecture-deep-summary.md
docs/tmp/verbose_extra_context/bootstrap-cluster-deep-summary.md
docs/tmp/verbose_extra_context/check-cluster-health-api-and-state.md
docs/tmp/verbose_extra_context/check-cluster-health-cli-overview.md
docs/tmp/verbose_extra_context/check-cluster-health-runtime-evidence.md
docs/tmp/verbose_extra_context/cli-surface-summary.md
docs/tmp/verbose_extra_context/cluster-start-command.md
docs/tmp/verbose_extra_context/configure-tls-extra-context.md
docs/tmp/verbose_extra_context/debug-cluster-issues-extra-context.md
docs/tmp/verbose_extra_context/failure-modes-deep-summary.md
docs/tmp/verbose_extra_context/handle-primary-failure-deep-summary.md
docs/tmp/verbose_extra_context/http-api-deep-summary.md
docs/tmp/verbose_extra_context/introduction-extra-context.md
docs/tmp/verbose_extra_context/leader-check-command.md
docs/tmp/verbose_extra_context/observing-failover-deep-summary.md
docs/tmp/verbose_extra_context/perform-switchover-deep-summary.md
docs/tmp/verbose_extra_context/pgtuskmaster-cli-deep-summary.md
docs/tmp/verbose_extra_context/run-tests-extra-context.md
docs/tmp/verbose_extra_context/runtime-config-deep-summary.md
docs/tmp/verbose_extra_context/runtime-config-summary.md


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
tokio = { version = "1.44.1", features = ["sync", "rt", "rt-multi-thread", "macros", "time", "process", "net", "io-util", "fs"] }
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


===== src/test_harness/mod.rs =====
use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

pub mod auth;
pub mod binaries;
pub mod etcd3;
pub mod ha_e2e;
pub mod namespace;
pub mod net_proxy;
pub mod pg16;
pub mod ports;
pub mod provenance;
pub mod runtime_config;
pub mod signals;
pub mod tls;

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("spawn failed for {binary}: {source}")]
    SpawnFailure {
        binary: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{component} did not become ready within {timeout:?}")]
    StartupTimeout {
        component: &'static str,
        timeout: Duration,
    },
    #[error("{component} exited before readiness with status {status}")]
    EarlyExit {
        component: &'static str,
        status: std::process::ExitStatus,
    },
    #[error("{component} did not exit within {timeout:?}")]
    ShutdownTimeout {
        component: &'static str,
        timeout: Duration,
    },
    #[error("stale path exists: {path}")]
    StalePath { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::ports::allocate_ports;
    use crate::test_harness::HarnessError;

    #[test]
    fn concurrent_namespace_and_port_allocations_are_isolated() -> Result<(), HarnessError> {
        let mut namespaces = Vec::new();
        let mut reservations = Vec::new();

        for idx in 0..12_u32 {
            let guard = NamespaceGuard::new(&format!("isolation-{idx}"))?;
            let namespace = guard.namespace()?.clone();
            let reservation = allocate_ports(3)?;
            namespaces.push((guard, namespace));
            reservations.push(reservation);
        }

        let mut all_ns = std::collections::BTreeSet::new();
        for (_, ns) in &namespaces {
            assert!(all_ns.insert(ns.id.clone()), "duplicate namespace id");
        }

        let mut all_ports = std::collections::BTreeSet::new();
        for reservation in &reservations {
            for port in reservation.as_slice() {
                assert!(all_ports.insert(*port), "duplicate allocated port");
            }
        }

        Ok(())
    }
}


===== tests/ha_multi_node_failover.rs =====
#[path = "ha/support/multi_node.rs"]
mod multi_node;
#[path = "ha/support/observer.rs"]
mod observer;

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_unassisted_failover_sql_consistency(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_unassisted_failover_sql_consistency().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_planned_switchover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_no_quorum_enters_failsafe_strict_all_nodes(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_no_quorum_enters_failsafe_strict_all_nodes().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    multi_node::e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity().await
}


===== tests/bdd_api_http.rs =====
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pgtuskmaster_rust::{
    api::worker::ApiWorkerCtx,
    config::{ApiAuthConfig, ApiRoleTokensConfig, RuntimeConfig},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, UnixMillis, WorkerError},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Default)]
struct RecordingStore {
    writes: Arc<Mutex<Vec<(String, String)>>>,
    deletes: Arc<Mutex<Vec<String>>>,
    kv: Arc<Mutex<std::collections::BTreeMap<String, String>>>,
}

impl RecordingStore {
    fn writes(&self) -> Result<Vec<(String, String)>, WorkerError> {
        let guard = self
            .writes
            .lock()
            .map_err(|_| WorkerError::Message("writes lock poisoned".to_string()))?;
        Ok(guard.clone())
    }

    fn deletes(&self) -> Result<Vec<String>, WorkerError> {
        let guard = self
            .deletes
            .lock()
            .map_err(|_| WorkerError::Message("deletes lock poisoned".to_string()))?;
        Ok(guard.clone())
    }
}

impl DcsStore for RecordingStore {
    fn healthy(&self) -> bool {
        true
    }

    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
        let guard = self
            .kv
            .lock()
            .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
        Ok(guard.get(path).cloned())
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        {
            let mut guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            guard.insert(path.to_string(), value.clone());
        }
        let mut guard = self
            .writes
            .lock()
            .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
        guard.push((path.to_string(), value));
        Ok(())
    }

    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
        {
            let mut guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            if guard.contains_key(path) {
                return Ok(false);
            }
            guard.insert(path.to_string(), value.clone());
        }
        let mut guard = self
            .writes
            .lock()
            .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
        guard.push((path.to_string(), value));
        Ok(true)
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        {
            let mut guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            guard.remove(path);
        }
        let mut guard = self
            .deletes
            .lock()
            .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?;
        guard.push(path.to_string());
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config(auth_token: Option<String>) -> RuntimeConfig {
    let auth = match auth_token {
        Some(token) => ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(token.clone()),
            admin_token: Some(token),
        }),
        None => ApiAuthConfig::Disabled,
    };

    pgtuskmaster_rust::test_harness::runtime_config::RuntimeConfigBuilder::new()
        .with_api_listen_addr("127.0.0.1:0")
        .with_api_auth(auth)
        .build()
}

const HEADER_LIMIT: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 256 * 1024;
const MAX_RESPONSE_BYTES: usize = HEADER_LIMIT + MAX_BODY_BYTES;
const IO_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
struct TestHttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers.iter().find_map(|(k, v)| {
        if k.eq_ignore_ascii_case(name) {
            Some(v.as_str())
        } else {
            None
        }
    })
}

#[derive(Debug)]
struct ParsedHttpHead {
    status_code: u16,
    headers: Vec<(String, String)>,
    content_length: usize,
    body_start: usize,
}

fn parse_http_response_head(raw: &[u8], header_end: usize) -> Result<ParsedHttpHead, WorkerError> {
    let head = raw.get(..header_end).ok_or_else(|| {
        WorkerError::Message("response header end offset out of bounds".to_string())
    })?;

    let status_line_end = head
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| WorkerError::Message("response missing status line".to_string()))?;

    let status_line_bytes = head.get(..status_line_end).ok_or_else(|| {
        WorkerError::Message("response status line offset out of bounds".to_string())
    })?;
    let status_line = std::str::from_utf8(status_line_bytes)
        .map_err(|err| WorkerError::Message(format!("response status line not utf8: {err}")))?;

    let mut status_parts = status_line.split_whitespace();
    let http_version = status_parts.next().ok_or_else(|| {
        WorkerError::Message("response status line missing http version".to_string())
    })?;
    if http_version != "HTTP/1.1" {
        return Err(WorkerError::Message(format!(
            "unexpected http version in response: {http_version}"
        )));
    }
    let status_str = status_parts.next().ok_or_else(|| {
        WorkerError::Message("response status line missing status code".to_string())
    })?;
    if status_str.len() != 3 || !status_str.bytes().all(|b| b.is_ascii_digit()) {
        return Err(WorkerError::Message(format!(
            "response status code must be 3 digits, got: {status_str}"
        )));
    }
    let status_code = status_str
        .parse::<u16>()
        .map_err(|err| WorkerError::Message(format!("response status code parse failed: {err}")))?;
    if !(100..=599).contains(&status_code) {
        return Err(WorkerError::Message(format!(
            "response status code out of range: {status_code}"
        )));
    }

    let header_text = head
        .get(status_line_end + 2..)
        .ok_or_else(|| WorkerError::Message("response header offset out of bounds".to_string()))?;
    let header_text = std::str::from_utf8(header_text)
        .map_err(|err| WorkerError::Message(format!("response headers not utf8: {err}")))?;

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length: Option<usize> = None;
    for line in header_text.split("\r\n") {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line.split_once(':').ok_or_else(|| {
            WorkerError::Message(format!(
                "invalid response header line (missing ':'): {line}"
            ))
        })?;
        let name = name.trim();
        let value = value.trim();
        headers.push((name.to_string(), value.to_string()));

        if name.eq_ignore_ascii_case("Content-Length") {
            if content_length.is_some() {
                return Err(WorkerError::Message(
                    "response contains multiple Content-Length headers".to_string(),
                ));
            }
            let parsed = value.parse::<usize>().map_err(|err| {
                WorkerError::Message(format!("response Content-Length parse failed: {err}"))
            })?;
            content_length = Some(parsed);
        }
    }

    let content_length = content_length.ok_or_else(|| {
        WorkerError::Message("response missing Content-Length header".to_string())
    })?;

    let body_start = header_end
        .checked_add(4)
        .ok_or_else(|| WorkerError::Message("response body offset overflow".to_string()))?;

    Ok(ParsedHttpHead {
        status_code,
        headers,
        content_length,
        body_start,
    })
}

async fn read_http_response_framed(
    stream: &mut (impl AsyncRead + Unpin),
    timeout: Duration,
) -> Result<TestHttpResponse, WorkerError> {
    let response = tokio::time::timeout(timeout, async {
        let mut raw: Vec<u8> = Vec::new();
        let mut scratch = [0u8; 4096];

        let mut parsed_head: Option<ParsedHttpHead> = None;
        let mut expected_total_len: Option<usize> = None;

        loop {
            if let Some(expected) = expected_total_len {
                if raw.len() == expected {
                    let parsed = parsed_head.ok_or_else(|| {
                        WorkerError::Message("response framing parsed without header".to_string())
                    })?;
                    let body = raw
                        .get(parsed.body_start..expected)
                        .ok_or_else(|| {
                            WorkerError::Message(
                                "response body slice out of bounds after framing".to_string(),
                            )
                        })?
                        .to_vec();
                    return Ok(TestHttpResponse {
                        status_code: parsed.status_code,
                        headers: parsed.headers,
                        body,
                    });
                }
                if raw.len() > expected {
                    return Err(WorkerError::Message(format!(
                        "response exceeded expected length (expected {expected} bytes, got {})",
                        raw.len()
                    )));
                }
            } else {
                if raw.len() > HEADER_LIMIT {
                    return Err(WorkerError::Message(format!(
                        "response headers exceeded limit of {HEADER_LIMIT} bytes"
                    )));
                }

                if let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = parse_http_response_head(&raw, header_end)?;
                    if head.content_length > MAX_BODY_BYTES {
                        return Err(WorkerError::Message(format!(
                            "response body exceeded limit of {MAX_BODY_BYTES} bytes (Content-Length={})",
                            head.content_length
                        )));
                    }
                    let expected = head.body_start.checked_add(head.content_length).ok_or_else(|| {
                        WorkerError::Message("response total length overflow".to_string())
                    })?;
                    if expected > MAX_RESPONSE_BYTES {
                        return Err(WorkerError::Message(format!(
                            "response exceeded limit of {MAX_RESPONSE_BYTES} bytes (expected {expected})"
                        )));
                    }
                    parsed_head = Some(head);
                    expected_total_len = Some(expected);
                    continue;
                }
            }

            let n = stream.read(&mut scratch).await.map_err(|err| {
                WorkerError::Message(format!("client read failed: {err}"))
            })?;
            if n == 0 {
                return Err(WorkerError::Message(format!(
                    "unexpected eof while reading response (read {} bytes so far)",
                    raw.len()
                )));
            }

            let new_len = raw.len().checked_add(n).ok_or_else(|| {
                WorkerError::Message("response length overflow while reading".to_string())
            })?;
            if new_len > MAX_RESPONSE_BYTES {
                return Err(WorkerError::Message(format!(
                    "response exceeded limit of {MAX_RESPONSE_BYTES} bytes while reading (would reach {new_len})"
                )));
            }
            raw.extend_from_slice(&scratch[..n]);
        }
    })
    .await;

    match response {
        Ok(inner) => inner,
        Err(_) => Err(WorkerError::Message(format!(
            "timed out reading framed http response after {}s",
            timeout.as_secs()
        ))),
    }
}

#[test]
fn bdd_http_parser_rejects_four_digit_status_code() -> Result<(), WorkerError> {
    let raw = b"HTTP/1.1 1200 OK\r\nContent-Length: 0\r\n\r\n";
    let header_end = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| {
            WorkerError::Message("synthetic response missing header terminator".to_string())
        })?;

    let parsed = parse_http_response_head(raw, header_end);
    if parsed.is_ok() {
        return Err(WorkerError::Message(
            "expected parser to reject 4-digit http status code, but it accepted it".to_string(),
        ));
    }
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_post_switchover_writes_dcs_key() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let body = br#"{"requested_by":"node-a"}"#;
    let request = format!(
        "POST /switchover HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write header failed: {err}")))?;
    client
        .write_all(body)
        .await
        .map_err(|err| WorkerError::Message(format!("client write body failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let response = read_http_response_framed(&mut client, IO_TIMEOUT).await?;
    assert_eq!(response.status_code, 202);
    let connection = header_value(&response.headers, "Connection")
        .ok_or_else(|| WorkerError::Message("response missing Connection header".to_string()))?;
    if connection != "close" {
        return Err(WorkerError::Message(format!(
            "expected Connection: close, got: {connection}"
        )));
    }
    let decoded: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|err| WorkerError::Message(format!("decode response json failed: {err}")))?;
    assert_eq!(decoded["accepted"], true);

    let writes = store.writes()?;
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].0, "/scope-a/switchover");
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_removed_ha_leader_routes_and_ha_state_contract() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut post_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let post_body = br#"{"member_id":"node-b"}"#;
    let post_request = format!(
        "POST /ha/leader HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        post_body.len()
    );
    post_client
        .write_all(post_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("post write header failed: {err}")))?;
    post_client
        .write_all(post_body)
        .await
        .map_err(|err| WorkerError::Message(format!("post write body failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let post_response = read_http_response_framed(&mut post_client, IO_TIMEOUT).await?;
    assert_eq!(post_response.status_code, 404);

    let mut delete_leader_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    delete_leader_client
        .write_all(b"DELETE /ha/leader HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .map_err(|err| WorkerError::Message(format!("delete leader write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let delete_leader_response =
        read_http_response_framed(&mut delete_leader_client, IO_TIMEOUT).await?;
    assert_eq!(delete_leader_response.status_code, 404);

    let mut delete_switchover_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    delete_switchover_client
        .write_all(
            b"DELETE /ha/switchover HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .await
        .map_err(|err| WorkerError::Message(format!("delete switchover write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let delete_switchover_response =
        read_http_response_framed(&mut delete_switchover_client, IO_TIMEOUT).await?;
    assert_eq!(delete_switchover_response.status_code, 202);

    let mut state_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    state_client
        .write_all(b"GET /ha/state HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .map_err(|err| WorkerError::Message(format!("state write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let state_response = read_http_response_framed(&mut state_client, IO_TIMEOUT).await?;
    assert_eq!(state_response.status_code, 503);
    let state_text = String::from_utf8(state_response.body)
        .map_err(|err| WorkerError::Message(format!("state body not utf8: {err}")))?;
    assert!(state_text.contains("snapshot unavailable"));

    let writes = store.writes()?;
    assert_eq!(writes.len(), 0);
    let deletes = store.deletes()?;
    assert_eq!(deletes, vec!["/scope-a/switchover"]);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_removed_ha_leader_routes_require_legacy_auth_token() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(Some("secret".to_string()));
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut denied_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let body = br#"{"member_id":"node-a"}"#;
    let denied_request = format!(
        "POST /ha/leader HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    denied_client
        .write_all(denied_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("denied write header failed: {err}")))?;
    denied_client
        .write_all(body)
        .await
        .map_err(|err| WorkerError::Message(format!("denied write body failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let denied_response = read_http_response_framed(&mut denied_client, IO_TIMEOUT).await?;
    assert_eq!(denied_response.status_code, 401);

    let mut allowed_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let allowed_request = format!(
        "POST /ha/leader HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: Bearer secret\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    allowed_client
        .write_all(allowed_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("allowed write header failed: {err}")))?;
    allowed_client
        .write_all(body)
        .await
        .map_err(|err| WorkerError::Message(format!("allowed write body failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let allowed_response = read_http_response_framed(&mut allowed_client, IO_TIMEOUT).await?;
    assert_eq!(allowed_response.status_code, 404);

    let mut state_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    state_client
        .write_all(
            b"GET /ha/state HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: Bearer secret\r\n\r\n",
        )
        .await
        .map_err(|err| WorkerError::Message(format!("state write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let state_response = read_http_response_framed(&mut state_client, IO_TIMEOUT).await?;
    assert_eq!(state_response.status_code, 503);

    let writes = store.writes()?;
    assert_eq!(writes.len(), 0);
    let deletes = store.deletes()?;
    assert_eq!(deletes.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_get_fallback_cluster_returns_name() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let request = "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let response = read_http_response_framed(&mut client, IO_TIMEOUT).await?;
    assert_eq!(response.status_code, 200);
    let decoded: serde_json::Value = serde_json::from_slice(&response.body)
        .map_err(|err| WorkerError::Message(format!("decode response json failed: {err}")))?;
    assert_eq!(decoded["name"], "cluster-a");
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_missing_header() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(Some("secret".to_string()));
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let request = "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let response = read_http_response_framed(&mut client, IO_TIMEOUT).await?;
    assert_eq!(response.status_code, 401);
    let writes = store.writes()?;
    assert_eq!(writes.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_debug_routes_expose_ui_and_verbose_contracts() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut ui_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let ui_request = "GET /debug/ui HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    ui_client
        .write_all(ui_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("ui write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let ui_response = read_http_response_framed(&mut ui_client, IO_TIMEOUT).await?;
    assert_eq!(ui_response.status_code, 200);
    let ui_html = String::from_utf8(ui_response.body)
        .map_err(|err| WorkerError::Message(format!("ui body not utf8: {err}")))?;
    assert!(ui_html.contains("id=\"meta-panel\""));
    assert!(ui_html.contains("/debug/verbose"));

    let mut verbose_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let verbose_request =
        "GET /debug/verbose HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    verbose_client
        .write_all(verbose_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("verbose write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let verbose_response = read_http_response_framed(&mut verbose_client, IO_TIMEOUT).await?;
    assert_eq!(verbose_response.status_code, 503);
    let verbose_text = String::from_utf8(verbose_response.body)
        .map_err(|err| WorkerError::Message(format!("verbose body not utf8: {err}")))?;
    assert!(verbose_text.contains("snapshot unavailable"));
    Ok(())
}


===== src/worker_contract_tests.rs =====
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{
    mpsc::{self, RecvTimeoutError},
    Arc, Mutex,
};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    api::{HaPhaseResponse, HaStateResponse},
    config::RuntimeConfig,
    dcs::state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx, SystemSnapshot},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::{
        decision::HaDecision,
        state::{HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, WorldSnapshot},
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, PgInfoWorkerCtx, Readiness, SqlStatus},
    process::{
        state::{JobOutcome, ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker as process_worker,
    },
    state::{
        new_state_channel, ClusterName, JobId, MemberId, UnixMillis, Version, Versioned,
        WorkerError, WorkerStatus,
    },
};

const CONTRACT_STORE_RELEASE_TIMEOUT: Duration = Duration::from_secs(5);
const CONTRACT_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);
const DEBUG_API_TEST_POLL_INTERVAL: Duration = Duration::from_millis(5);
const CONTRACT_BLOCKING_START_TIMEOUT: Duration = Duration::from_secs(1);
const CONTRACT_API_RESPONSIVE_DEADLINE: Duration = Duration::from_millis(500);

#[derive(Default)]
struct ContractStore;

impl DcsStore for ContractStore {
    fn healthy(&self) -> bool {
        true
    }

    fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(None)
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn put_path_if_absent(&mut self, _path: &str, _value: String) -> Result<bool, DcsStoreError> {
        Ok(true)
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

struct BlockingAcquireStore {
    acquire_started: Arc<Mutex<Option<mpsc::Sender<()>>>>,
    acquire_release: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl BlockingAcquireStore {
    fn new() -> (Self, mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        (
            Self {
                acquire_started: Arc::new(Mutex::new(Some(started_tx))),
                acquire_release: Arc::new(Mutex::new(release_rx)),
            },
            started_rx,
            release_tx,
        )
    }
}

impl DcsStore for BlockingAcquireStore {
    fn healthy(&self) -> bool {
        true
    }

    fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(None)
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn put_path_if_absent(&mut self, _path: &str, _value: String) -> Result<bool, DcsStoreError> {
        let mut started_guard = self
            .acquire_started
            .lock()
            .map_err(|_| DcsStoreError::Io("acquire started lock poisoned".to_string()))?;
        if let Some(tx) = started_guard.take() {
            tx.send(())
                .map_err(|_| DcsStoreError::Io("acquire started signal failed".to_string()))?;
        }
        drop(started_guard);

        let release_guard = self
            .acquire_release
            .lock()
            .map_err(|_| DcsStoreError::Io("acquire release lock poisoned".to_string()))?;
        match release_guard.recv_timeout(CONTRACT_STORE_RELEASE_TIMEOUT) {
            Ok(()) => Ok(true),
            Err(RecvTimeoutError::Timeout) => Err(DcsStoreError::Io(
                "acquire release unblock timed out".to_string(),
            )),
            Err(RecvTimeoutError::Disconnected) => Err(DcsStoreError::Io(
                "acquire release unblock disconnected".to_string(),
            )),
        }
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config() -> RuntimeConfig {
    crate::test_harness::runtime_config::sample_runtime_config()
}

fn sample_pg_state() -> PgInfoState {
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

fn sample_primary_pg_state() -> PgInfoState {
    PgInfoState::Primary {
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
        wal_lsn: crate::state::WalLsn(1),
        slots: Vec::new(),
    }
}

fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
    DcsState {
        worker: WorkerStatus::Starting,
        trust: DcsTrust::NotTrusted,
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg,
            init_lock: None,
        },
        last_refresh_at: None,
    }
}

fn sample_dcs_state_with_trust(cfg: RuntimeConfig, trust: DcsTrust) -> DcsState {
    DcsState {
        trust,
        ..sample_dcs_state(cfg)
    }
}

fn sample_process_state() -> ProcessState {
    ProcessState::Idle {
        worker: WorkerStatus::Starting,
        last_outcome: None,
    }
}

fn sample_ha_state() -> HaState {
    HaState {
        worker: WorkerStatus::Starting,
        phase: HaPhase::Init,
        tick: 0,
        decision: HaDecision::EnterFailSafe {
            release_leader_lease: false,
        },
    }
}

async fn get_ha_state_via_tcp(addr: SocketAddr) -> Result<HaStateResponse, WorkerError> {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("api connect failed: {err}")))?;
    stream
        .write_all(b"GET /ha/state HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .map_err(|err| WorkerError::Message(format!("api request write failed: {err}")))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .await
        .map_err(|err| WorkerError::Message(format!("api response read failed: {err}")))?;
    let text = String::from_utf8(raw)
        .map_err(|err| WorkerError::Message(format!("api response utf8 failed: {err}")))?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| WorkerError::Message("api response missing header separator".to_string()))?;
    if !head.starts_with("HTTP/1.1 200") {
        return Err(WorkerError::Message(format!(
            "api returned unexpected response: {head}"
        )));
    }
    serde_json::from_str(body)
        .map_err(|err| WorkerError::Message(format!("api response decode failed: {err}")))
}

#[test]
fn required_state_types_exist() {
    let _process_state: Option<ProcessState> = None;
    let _process_job_kind: Option<ProcessJobKind> = None;
    let _job_outcome: Option<JobOutcome> = None;

    let _ha_phase: Option<HaPhase> = None;
    let _ha_state: Option<HaState> = None;
    let _world_snapshot: Option<WorldSnapshot> = None;

    let _system_snapshot: Option<SystemSnapshot> = None;
}

#[test]
fn worker_contract_symbols_exist() {
    let _ = crate::pginfo::worker::run;
    let _ = crate::pginfo::worker::step_once;

    let _ = crate::dcs::worker::run;
    let _ = crate::dcs::worker::step_once;

    let _ = process_worker::run;
    let _ = process_worker::step_once;

    let _ = crate::ha::worker::run;
    let _ = crate::ha::worker::step_once;

    let _ = crate::api::worker::run;
    let _ = crate::api::worker::step_once;

    let _ = crate::debug_api::worker::run;
    let _ = crate::debug_api::worker::step_once;
}

#[tokio::test(flavor = "current_thread")]
async fn step_once_contracts_are_callable() -> Result<(), WorkerError> {
    let self_member_id = MemberId("node-a".to_string());

    let initial_pg = sample_pg_state();
    let (publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
    let mut pg_ctx = PgInfoWorkerCtx {
        self_id: self_member_id.clone(),
        postgres_conninfo: crate::pginfo::state::PgConnInfo {
            host: "127.0.0.1".to_string(),
            port: 1,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: None,
            ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
            options: None,
        },
        poll_interval: CONTRACT_WORKER_POLL_INTERVAL,
        publisher,
        log: crate::logging::LogHandle::null(),
        last_emitted_sql_status: None,
    };
    crate::pginfo::worker::step_once(&mut pg_ctx).await?;
    let pg_latest = pg_subscriber.latest();
    assert_eq!(pg_latest.version, Version(1));
    assert!(matches!(
        &pg_latest.value,
        PgInfoState::Unknown { common }
            if common.worker == WorkerStatus::Running && common.sql == SqlStatus::Unreachable
    ));

    let initial_dcs = sample_dcs_state(sample_runtime_config());
    let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));
    let dcs_pg_subscriber = pg_subscriber.clone();
    let mut dcs_ctx = DcsWorkerCtx {
        self_id: self_member_id.clone(),
        scope: "scope-a".to_string(),
        poll_interval: CONTRACT_WORKER_POLL_INTERVAL,
        local_postgres_host: sample_runtime_config().postgres.listen_host.clone(),
        local_postgres_port: sample_runtime_config().postgres.listen_port,
        pg_subscriber: dcs_pg_subscriber,
        publisher: dcs_publisher,
        store: Box::new(ContractStore),
        log: crate::logging::LogHandle::null(),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        },
        last_published_pg_version: None,
        last_emitted_store_healthy: None,
        last_emitted_trust: None,
    };
    crate::dcs::worker::step_once(&mut dcs_ctx).await?;
    let dcs_latest = dcs_subscriber.latest();
    assert_eq!(dcs_latest.version, Version(1));
    assert!(dcs_latest.value.last_refresh_at.is_some());
    assert_eq!(dcs_ctx.last_published_pg_version, Some(pg_latest.version));
    assert!(dcs_ctx.cache.members.contains_key(&self_member_id));

    let initial_process = sample_process_state();
    let (process_publisher, process_subscriber) = new_state_channel(initial_process, UnixMillis(1));
    let (_process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut process_ctx = ProcessWorkerCtx::contract_stub(
        sample_runtime_config().process.clone(),
        process_publisher,
        process_rx,
    );
    process_worker::step_once(&mut process_ctx).await?;
    assert!(matches!(&process_ctx.state, ProcessState::Idle { .. }));
    assert!(process_ctx.state.running_job_id().is_none());
    assert!(matches!(
        &process_ctx.state,
        ProcessState::Idle {
            last_outcome: None,
            ..
        }
    ));
    let process_latest = process_subscriber.latest();
    assert_eq!(process_latest.version, Version(0));
    assert!(matches!(
        &process_latest.value,
        ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None
        }
    ));

    let runtime_cfg = sample_runtime_config();
    let initial_ha = sample_ha_state();
    let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha, UnixMillis(1));
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(runtime_cfg.clone(), UnixMillis(1));
    let api_cfg_subscriber = cfg_subscriber.clone();
    let debug_cfg_subscriber = cfg_subscriber.clone();
    let (_ha_pg_publisher, ha_pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
    let debug_pg_subscriber = ha_pg_subscriber.clone();
    let (_ha_dcs_publisher, ha_dcs_subscriber) =
        new_state_channel(sample_dcs_state(runtime_cfg.clone()), UnixMillis(1));
    let debug_dcs_subscriber = ha_dcs_subscriber.clone();
    let (_ha_process_publisher, ha_process_subscriber) =
        new_state_channel(sample_process_state(), UnixMillis(1));
    let debug_process_subscriber = ha_process_subscriber.clone();
    let (ha_process_tx, _ha_process_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber,
        pg_subscriber: ha_pg_subscriber,
        dcs_subscriber: ha_dcs_subscriber,
        process_subscriber: ha_process_subscriber,
        process_inbox: ha_process_tx,
        dcs_store: Box::new(ContractStore),
        scope: "scope-a".to_string(),
        self_id: self_member_id.clone(),
    });
    crate::ha::worker::step_once(&mut ha_ctx).await?;
    assert_eq!(ha_ctx.state.phase, HaPhase::FailSafe);
    assert_eq!(ha_ctx.state.tick, 1);
    assert_eq!(ha_ctx.state.worker, WorkerStatus::Running);
    let ha_latest = ha_subscriber.latest();
    assert_eq!(ha_latest.version, Version(1));
    assert_eq!(ha_latest.value, ha_ctx.state);
    let debug_ha_subscriber = ha_subscriber.clone();

    let api_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("api bind failed: {err}")))?;
    let mut api_ctx = crate::api::worker::ApiWorkerCtx::contract_stub(
        api_listener,
        api_cfg_subscriber,
        Box::new(ContractStore),
    );
    let api_addr_before = api_ctx.local_addr()?;
    crate::api::worker::step_once(&mut api_ctx).await?;
    let api_addr_after = api_ctx.local_addr()?;
    assert_eq!(api_addr_before, api_addr_after);

    let initial_debug_snapshot = SystemSnapshot {
        app: AppLifecycle::Starting,
        config: debug_cfg_subscriber.latest(),
        pg: debug_pg_subscriber.latest(),
        dcs: debug_dcs_subscriber.latest(),
        process: debug_process_subscriber.latest(),
        ha: debug_ha_subscriber.latest(),
        generated_at: UnixMillis(1),
        sequence: 0,
        changes: Vec::new(),
        timeline: Vec::new(),
    };
    let (debug_publisher, debug_subscriber) =
        new_state_channel(initial_debug_snapshot, UnixMillis(1));
    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: debug_cfg_subscriber,
        pg_subscriber: debug_pg_subscriber,
        dcs_subscriber: debug_dcs_subscriber,
        process_subscriber: debug_process_subscriber,
        ha_subscriber: debug_ha_subscriber,
    });
    crate::debug_api::worker::step_once(&mut debug_ctx).await?;
    let debug_latest = debug_subscriber.latest();
    assert_eq!(debug_latest.version, Version(1));
    assert_eq!(debug_latest.value.app, AppLifecycle::Starting);
    assert_eq!(debug_latest.value.config.version, Version(0));
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ha_state_api_stays_responsive_while_ha_attempt_leadership_blocks(
) -> Result<(), WorkerError> {
    let runtime_cfg = sample_runtime_config();
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(runtime_cfg.clone(), UnixMillis(1));
    let (_pg_publisher, pg_subscriber) =
        new_state_channel(sample_primary_pg_state(), UnixMillis(1));
    let (_dcs_publisher, dcs_subscriber) = new_state_channel(
        sample_dcs_state_with_trust(runtime_cfg.clone(), DcsTrust::FullQuorum),
        UnixMillis(1),
    );
    let (_process_publisher, process_subscriber) =
        new_state_channel(sample_process_state(), UnixMillis(1));
    let (ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

    let initial_snapshot = build_snapshot(
        &DebugSnapshotCtx {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
        },
        UnixMillis(1),
        0,
        &[],
        &[],
    );
    let (debug_publisher, debug_subscriber) = new_state_channel(initial_snapshot, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("api bind failed: {err}")))?;
    let mut api_ctx = crate::api::worker::ApiWorkerCtx::contract_stub(
        listener,
        cfg_subscriber.clone(),
        Box::new(ContractStore),
    );
    api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
    let api_addr = api_ctx.local_addr()?;

    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        ha_subscriber: ha_subscriber.clone(),
    });
    debug_ctx.app = AppLifecycle::Running;
    debug_ctx.poll_interval = DEBUG_API_TEST_POLL_INTERVAL;

    let (process_tx, _process_rx) = tokio::sync::mpsc::unbounded_channel();
    let (store, acquire_started_rx, acquire_release_tx) = BlockingAcquireStore::new();
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber,
        pg_subscriber,
        dcs_subscriber,
        process_subscriber,
        process_inbox: process_tx,
        dcs_store: Box::new(store),
        scope: "scope-a".to_string(),
        self_id: MemberId("node-a".to_string()),
    });
    ha_ctx.state = HaState {
        worker: WorkerStatus::Running,
        phase: HaPhase::FailSafe,
        tick: 0,
        decision: HaDecision::NoChange,
    };

    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            let api_handle =
                tokio::task::spawn_local(async move { crate::api::worker::run(api_ctx).await });
            let debug_handle =
                tokio::task::spawn_local(
                    async move { crate::debug_api::worker::run(debug_ctx).await },
                );
            let ha_handle = tokio::task::spawn_local(async move {
                let result = crate::ha::worker::step_once(&mut ha_ctx).await;
                (ha_ctx, result)
            });

            let started_result = tokio::task::spawn_blocking(move || {
                acquire_started_rx.recv_timeout(CONTRACT_BLOCKING_START_TIMEOUT)
            })
            .await
            .map_err(|err| WorkerError::Message(format!("blocking wait join failed: {err}")))?;
            match started_result {
                Ok(()) => {}
                Err(err) => {
                    api_handle.abort();
                    debug_handle.abort();
                    ha_handle.abort();
                    return Err(WorkerError::Message(format!(
                        "blocking acquire-leader path did not start: {err}"
                    )));
                }
            }

            let deadline = tokio::time::Instant::now() + CONTRACT_API_RESPONSIVE_DEADLINE;
            let observed = loop {
                match get_ha_state_via_tcp(api_addr).await {
                    Ok(state)
                        if state.ha_phase == HaPhaseResponse::Primary && state.ha_tick == 1 =>
                    {
                        break state;
                    }
                    Ok(_state) => {}
                    Err(_err) => {}
                }
                if tokio::time::Instant::now() >= deadline {
                    api_handle.abort();
                    debug_handle.abort();
                    ha_handle.abort();
                    return Err(WorkerError::Message(
                        "timed out waiting for responsive /ha/state".to_string(),
                    ));
                }
                tokio::time::sleep(CONTRACT_WORKER_POLL_INTERVAL).await;
            };

            acquire_release_tx
                .send(())
                .map_err(|_| WorkerError::Message("acquire release signal failed".to_string()))?;
            let (ha_ctx, ha_result) = ha_handle
                .await
                .map_err(|err| WorkerError::Message(format!("ha step join failed: {err}")))?;
            ha_result?;

            api_handle.abort();
            debug_handle.abort();
            let _ = api_handle.await;
            let _ = debug_handle.await;

            assert_eq!(observed.ha_phase, HaPhaseResponse::Primary);
            assert!(observed.snapshot_sequence > 0);
            assert_eq!(ha_ctx.state.phase, HaPhase::Primary);
            Ok(())
        })
        .await
}

#[test]
fn snapshot_contract_type_compiles() {
    let cfg = sample_runtime_config();
    let pg = sample_pg_state();
    let dcs = sample_dcs_state(cfg.clone());
    let process = sample_process_state();
    let ha = sample_ha_state();

    let world = WorldSnapshot {
        config: Versioned::new(Version(1), UnixMillis(1), cfg.clone()),
        pg: Versioned::new(Version(1), UnixMillis(1), pg),
        dcs: Versioned::new(Version(1), UnixMillis(1), dcs),
        process: Versioned::new(Version(1), UnixMillis(1), process),
    };
    assert_eq!(world.config.version, Version(1));

    let debug_ctx = crate::debug_api::snapshot::DebugSnapshotCtx {
        app: crate::debug_api::snapshot::AppLifecycle::Running,
        config: Versioned::new(Version(2), UnixMillis(2), cfg),
        pg: Versioned::new(Version(2), UnixMillis(2), sample_pg_state()),
        dcs: Versioned::new(
            Version(2),
            UnixMillis(2),
            sample_dcs_state(sample_runtime_config()),
        ),
        process: Versioned::new(Version(2), UnixMillis(2), sample_process_state()),
        ha: Versioned::new(Version(2), UnixMillis(2), ha),
    };

    let system = crate::debug_api::snapshot::build_snapshot(&debug_ctx, UnixMillis(2), 0, &[], &[]);
    assert_eq!(system.config.version, Version(2));
    let _unused = ClusterName("cluster-a".to_string());
    let _job_id = JobId("job-1".to_string());
}


===== docs/tmp/verbose_extra_context/run-tests-extra-context.md =====
# Extra context for docs/src/how-to/run-tests.md

This repository has no published contributor testing guide yet, but the local build tooling and harness code provide enough factual material to explain how tests are run.

Canonical entrypoints from `Makefile`:

- `make check` runs `cargo check --all-targets`.
- `make test` runs `cargo nextest run --workspace --all-targets --profile default --no-fail-fast --no-tests fail`.
- `make test-long` runs `cargo nextest run --workspace --all-targets --profile ultra-long --no-fail-fast --no-tests fail`, then Docker Compose validation and two Docker smoke scripts.
- `make lint` includes `docs-lint`, the silent-error guard, and multiple `cargo clippy` passes.

Important prerequisites directly visible in the repo:

- `cargo-nextest` is required for `make test` and `make test-long`.
- `timeout` or `gtimeout` is required for the Makefile gates that use timeouts.
- Docker and the Docker Compose plugin are required for `make test-long` and the docker smoke/config targets.
- The test harness requires real binaries for etcd and PostgreSQL 16. Source paths such as `src/test_harness/binaries.rs`, `src/test_harness/provenance.rs`, and `src/test_harness/ha_e2e/startup.rs` show that real-binary attestations are enforced and that helper install scripts `./tools/install-etcd.sh` and `./tools/install-postgres16.sh` are the expected way to satisfy them.
- The dev image in `docker/Dockerfile.dev` installs `protobuf-compiler`, `pkg-config`, Node, npm, ripgrep, Python, and Rust tooling, which is a good indicator of the local toolchain expected by developers.

What kinds of tests exist, based on files requested by K2:

- `tests/ha_multi_node_failover.rs` is a top-level entry for HA end-to-end scenarios.
- `tests/ha/support/multi_node.rs` contains large multi-node scenarios, convergence waits, stress workloads, and API-driven switchovers.
- `tests/ha/support/observer.rs` contains split-brain detection and HA observation helpers.
- `tests/bdd_api_http.rs` contains behavior-driven HTTP API tests that use a sample runtime config and raw HTTP framing checks.
- `src/worker_contract_tests.rs` contains contract-style tests that assert required state types, debug API responsiveness, and worker coordination behavior.

External dependencies and runtime environment details that are supportable from source:

- Real HA scenarios depend on etcd and PostgreSQL 16 binaries, not mocks only.
- Some scenarios also rely on Docker Compose stacks under `docker/compose/`.
- The harness allocates ports dynamically and creates isolated namespaces, so parallel test execution is expected.
- The Makefile forces a shared target dir under `/tmp/pgtuskmaster_rust-target` and disables incremental builds by default for deterministic gates.

Environment variables and flags visible in source/docs:

- `PGTUSKMASTER_READ_TOKEN` and `PGTUSKMASTER_ADMIN_TOKEN` are CLI env fallbacks, but these are API auth inputs rather than test-runner controls.
- The repository evidence does not show a dedicated test-only environment variable contract for choosing subsets of tests; the primary documented knobs are the Makefile targets and ordinary cargo/nextest filtering.
- A draft should avoid inventing hidden test env vars that are not clearly present in source.

Execution time and resource expectations that can be stated factually:

- `make test-long` is intentionally expensive: the Makefile describes it as running ultra-long HA scenarios plus Docker Compose validation and smoke coverage.
- Timeouts in the Makefile show the intended rough scale:
  - cargo check gate timeout: 300 seconds
  - docs lint timeout: 120 seconds
  - clippy timeout: 1200 seconds
  - docker compose config timeout: 120 seconds
  - docker smoke single timeout: 600 seconds
  - docker smoke cluster timeout: 900 seconds
- HA scenario constants in `tests/ha/support/multi_node.rs` include timeouts up to 180 seconds for loaded failover and 300 seconds for a scenario budget, which further confirms that long-running integration tests are normal.

Guidance boundaries:

- The doc should describe the real prerequisites honestly.
- It should not claim tests are lightweight unit-only checks.
- It should not claim any test is optional when the codebase clearly treats real-binary validation as mandatory.
