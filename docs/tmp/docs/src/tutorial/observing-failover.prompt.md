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

docs/src/tutorial/observing-failover.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/perform-switchover.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/pgtuskmasterctl-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/first-ha-cluster.md


# current docs summary context

===== docs/src/SUMMARY.md =====
# Summary

# Tutorials
- [Tutorials]()
    - [First HA Cluster](tutorial/first-ha-cluster.md)

# How-To

- [How-To]()
    - [Check Cluster Health](how-to/check-cluster-health.md)
    - [Perform a Planned Switchover](how-to/perform-switchover.md)

# Explanation

- [Explanation]()
    - [Architecture](explanation/architecture.md)

# Reference

- [Reference]()
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
docs/draft/docs/src/how-to/bootstrap-cluster.md
docs/draft/docs/src/how-to/check-cluster-health.md
docs/draft/docs/src/how-to/check-cluster-health.revised.md
docs/draft/docs/src/how-to/handle-primary-failure.md
docs/draft/docs/src/how-to/perform-switchover.md
docs/draft/docs/src/how-to/perform-switchover.revised.md
docs/draft/docs/src/reference/cli-commands.md
docs/draft/docs/src/reference/cli-commands.revised.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.revised.md
docs/draft/docs/src/reference/cli.md
docs/draft/docs/src/reference/cli.revised.md
docs/draft/docs/src/reference/http-api.md
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
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/how-to/check-cluster-health.md
docs/src/how-to/perform-switchover.md
docs/src/reference/pgtuskmaster-cli.md
docs/src/reference/pgtuskmasterctl-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/tmp/docs/src/explanation/architecture.prompt.md
docs/tmp/docs/src/explanation/failure-modes.prompt.md
docs/tmp/docs/src/how-to/bootstrap-cluster.prompt.md
docs/tmp/docs/src/how-to/check-cluster-health.prompt.md
docs/tmp/docs/src/how-to/handle-primary-failure.prompt.md
docs/tmp/docs/src/how-to/perform-switchover.prompt.md
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
docs/tmp/verbose_extra_context/failure-modes-deep-summary.md
docs/tmp/verbose_extra_context/handle-primary-failure-deep-summary.md
docs/tmp/verbose_extra_context/http-api-deep-summary.md
docs/tmp/verbose_extra_context/leader-check-command.md
docs/tmp/verbose_extra_context/observing-failover-deep-summary.md
docs/tmp/verbose_extra_context/perform-switchover-deep-summary.md
docs/tmp/verbose_extra_context/pgtuskmaster-cli-deep-summary.md
docs/tmp/verbose_extra_context/runtime-config-deep-summary.md
docs/tmp/verbose_extra_context/runtime-config-summary.md


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


===== tests/ha_partition_isolation.rs =====
#[path = "ha/support/observer.rs"]
mod observer;
#[path = "ha/support/partition.rs"]
mod partition;

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_minority_isolation_no_split_brain_rejoin(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_minority_isolation_no_split_brain_rejoin().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_primary_isolation_failover_no_split_brain(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_primary_isolation_failover_no_split_brain().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_api_path_isolation_preserves_primary(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_api_path_isolation_preserves_primary().await
}

#[tokio::test(flavor = "current_thread")]
async fn e2e_partition_mixed_faults_heal_converges(
) -> Result<(), pgtuskmaster_rust::state::WorkerError> {
    partition::e2e_partition_mixed_faults_heal_converges().await
}


===== docker/compose/docker-compose.cluster.yml =====
services:
  etcd:
    image: ${ETCD_IMAGE}
    command:
      - etcd
      - --name=etcd
      - --data-dir=/etcd-data
      - --listen-client-urls=http://0.0.0.0:2379
      - --advertise-client-urls=http://etcd:2379
      - --listen-peer-urls=http://0.0.0.0:2380
      - --initial-advertise-peer-urls=http://etcd:2380
      - --initial-cluster=etcd=http://etcd:2380
      - --initial-cluster-state=new
    healthcheck:
      test: ["CMD", "etcdctl", "--endpoints=http://127.0.0.1:2379", "endpoint", "health"]
      interval: 5s
      timeout: 5s
      retries: 20
    networks:
      - pgtm-internal
    volumes:
      - etcd-cluster-data:/etcd-data

  node-a:
    image: ${PGTUSKMASTER_IMAGE}
    build:
      context: ../..
      dockerfile: docker/Dockerfile.prod
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-a
        target: /etc/pgtuskmaster/runtime.toml
      - source: common-pg-hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: common-pg-ident
        target: /etc/pgtuskmaster/pg_ident.conf
    secrets:
      - source: superuser-password
        target: postgres-superuser-password
      - source: replicator-password
        target: replicator-password
      - source: rewinder-password
        target: rewinder-password
    networks:
      - pgtm-internal
    ports:
      - "${PGTM_CLUSTER_NODE_A_API_PORT}:8080"
      - "${PGTM_CLUSTER_NODE_A_PG_PORT}:5432"
    volumes:
      - node-a-cluster-data:/var/lib/postgresql
      - node-a-cluster-logs:/var/log/pgtuskmaster

  node-b:
    image: ${PGTUSKMASTER_IMAGE}
    build:
      context: ../..
      dockerfile: docker/Dockerfile.prod
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-b
        target: /etc/pgtuskmaster/runtime.toml
      - source: common-pg-hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: common-pg-ident
        target: /etc/pgtuskmaster/pg_ident.conf
    secrets:
      - source: superuser-password
        target: postgres-superuser-password
      - source: replicator-password
        target: replicator-password
      - source: rewinder-password
        target: rewinder-password
    networks:
      - pgtm-internal
    ports:
      - "${PGTM_CLUSTER_NODE_B_API_PORT}:8080"
      - "${PGTM_CLUSTER_NODE_B_PG_PORT}:5432"
    volumes:
      - node-b-cluster-data:/var/lib/postgresql
      - node-b-cluster-logs:/var/log/pgtuskmaster

  node-c:
    image: ${PGTUSKMASTER_IMAGE}
    build:
      context: ../..
      dockerfile: docker/Dockerfile.prod
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-c
        target: /etc/pgtuskmaster/runtime.toml
      - source: common-pg-hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: common-pg-ident
        target: /etc/pgtuskmaster/pg_ident.conf
    secrets:
      - source: superuser-password
        target: postgres-superuser-password
      - source: replicator-password
        target: replicator-password
      - source: rewinder-password
        target: rewinder-password
    networks:
      - pgtm-internal
    ports:
      - "${PGTM_CLUSTER_NODE_C_API_PORT}:8080"
      - "${PGTM_CLUSTER_NODE_C_PG_PORT}:5432"
    volumes:
      - node-c-cluster-data:/var/lib/postgresql
      - node-c-cluster-logs:/var/log/pgtuskmaster

configs:
  runtime-node-a:
    file: ../configs/cluster/node-a/runtime.toml
  runtime-node-b:
    file: ../configs/cluster/node-b/runtime.toml
  runtime-node-c:
    file: ../configs/cluster/node-c/runtime.toml
  common-pg-hba:
    file: ../configs/common/pg_hba.conf
  common-pg-ident:
    file: ../configs/common/pg_ident.conf

secrets:
  superuser-password:
    file: ${PGTM_SECRET_SUPERUSER_FILE}
  replicator-password:
    file: ${PGTM_SECRET_REPLICATOR_FILE}
  rewinder-password:
    file: ${PGTM_SECRET_REWINDER_FILE}

networks:
  pgtm-internal:
    driver: bridge

volumes:
  etcd-cluster-data:
  node-a-cluster-data:
  node-a-cluster-logs:
  node-b-cluster-data:
  node-b-cluster-logs:
  node-c-cluster-data:
  node-c-cluster-logs:


===== src/debug_api/mod.rs =====
pub(crate) mod snapshot;
pub(crate) mod view;
pub(crate) mod worker;


===== src/debug_api/snapshot.rs =====
use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    ha::state::HaState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{UnixMillis, Version, Versioned},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AppLifecycle {
    Starting,
    Running,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SystemSnapshot {
    pub(crate) app: AppLifecycle,
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
    pub(crate) ha: Versioned<HaState>,
    pub(crate) generated_at: UnixMillis,
    pub(crate) sequence: u64,
    pub(crate) changes: Vec<DebugChangeEvent>,
    pub(crate) timeline: Vec<DebugTimelineEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DebugDomain {
    App,
    Config,
    PgInfo,
    Dcs,
    Process,
    Ha,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugChangeEvent {
    pub(crate) sequence: u64,
    pub(crate) at: UnixMillis,
    pub(crate) domain: DebugDomain,
    pub(crate) previous_version: Option<Version>,
    pub(crate) current_version: Option<Version>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugTimelineEntry {
    pub(crate) sequence: u64,
    pub(crate) at: UnixMillis,
    pub(crate) domain: DebugDomain,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugSnapshotCtx {
    pub(crate) app: AppLifecycle,
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
    pub(crate) ha: Versioned<HaState>,
}

pub(crate) fn build_snapshot(
    ctx: &DebugSnapshotCtx,
    now: UnixMillis,
    sequence: u64,
    changes: &[DebugChangeEvent],
    timeline: &[DebugTimelineEntry],
) -> SystemSnapshot {
    SystemSnapshot {
        app: ctx.app.clone(),
        config: ctx.config.clone(),
        pg: ctx.pg.clone(),
        dcs: ctx.dcs.clone(),
        process: ctx.process.clone(),
        ha: ctx.ha.clone(),
        generated_at: now,
        sequence,
        changes: changes.to_vec(),
        timeline: timeline.to_vec(),
    }
}


===== src/ha/decision.rs =====
use serde::{Deserialize, Serialize};

use crate::{
    dcs::state::{DcsTrust, MemberRole},
    pginfo::state::{PgInfoState, SqlStatus},
    process::{
        jobs::ActiveJobKind,
        state::{JobOutcome, ProcessState},
    },
    state::{MemberId, TimelineId},
};

use super::state::{HaPhase, WorldSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionFacts {
    pub(crate) self_member_id: MemberId,
    pub(crate) trust: DcsTrust,
    pub(crate) postgres_reachable: bool,
    pub(crate) postgres_primary: bool,
    pub(crate) leader_member_id: Option<MemberId>,
    pub(crate) active_leader_member_id: Option<MemberId>,
    pub(crate) available_primary_member_id: Option<MemberId>,
    pub(crate) switchover_requested_by: Option<MemberId>,
    pub(crate) i_am_leader: bool,
    pub(crate) has_other_leader_record: bool,
    pub(crate) has_available_other_leader: bool,
    pub(crate) rewind_required: bool,
    pub(crate) process_state: ProcessState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessActivity {
    Running,
    IdleNoOutcome,
    IdleSuccess,
    IdleFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhaseOutcome {
    pub(crate) next_phase: HaPhase,
    pub(crate) decision: HaDecision,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum HaDecision {
    #[default]
    NoChange,
    WaitForPostgres {
        start_requested: bool,
        leader_member_id: Option<MemberId>,
    },
    WaitForDcsTrust,
    AttemptLeadership,
    FollowLeader {
        leader_member_id: MemberId,
    },
    BecomePrimary {
        promote: bool,
    },
    StepDown(StepDownPlan),
    RecoverReplica {
        strategy: RecoveryStrategy,
    },
    FenceNode,
    ReleaseLeaderLease {
        reason: LeaseReleaseReason,
    },
    EnterFailSafe {
        release_leader_lease: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StepDownPlan {
    pub(crate) reason: StepDownReason,
    pub(crate) release_leader_lease: bool,
    pub(crate) clear_switchover: bool,
    pub(crate) fence: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum StepDownReason {
    Switchover,
    ForeignLeaderDetected { leader_member_id: MemberId },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum RecoveryStrategy {
    Rewind { leader_member_id: MemberId },
    BaseBackup { leader_member_id: MemberId },
    Bootstrap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LeaseReleaseReason {
    FencingComplete,
    PostgresUnreachable,
}

impl DecisionFacts {
    pub(crate) fn from_world(world: &WorldSnapshot) -> Self {
        let self_member_id = MemberId(world.config.value.cluster.member_id.clone());
        let leader_member_id = world
            .dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|record| record.member_id.clone());
        let active_leader_member_id = leader_member_id
            .clone()
            .filter(|leader_id| is_available_primary_leader(world, leader_id));
        let available_primary_member_id = active_leader_member_id.clone().or_else(|| {
            world
                .dcs
                .value
                .cache
                .members
                .values()
                .find(|member| {
                    member.member_id != self_member_id
                        && member.role == MemberRole::Primary
                        && member.sql == SqlStatus::Healthy
                })
                .map(|member| member.member_id.clone())
        });
        let i_am_leader = leader_member_id.as_ref() == Some(&self_member_id);
        let has_other_leader_record = leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);
        let has_available_other_leader = active_leader_member_id
            .as_ref()
            .map(|leader_id| leader_id != &self_member_id)
            .unwrap_or(false);

        Self {
            self_member_id,
            trust: world.dcs.value.trust.clone(),
            postgres_reachable: is_postgres_reachable(&world.pg.value),
            postgres_primary: is_local_primary(&world.pg.value),
            leader_member_id,
            active_leader_member_id: active_leader_member_id.clone(),
            available_primary_member_id: available_primary_member_id.clone(),
            switchover_requested_by: world
                .dcs
                .value
                .cache
                .switchover
                .as_ref()
                .map(|request| request.requested_by.clone()),
            i_am_leader,
            has_other_leader_record,
            has_available_other_leader,
            rewind_required: available_primary_member_id
                .as_ref()
                .map(|leader_id| should_rewind_from_leader(world, leader_id))
                .unwrap_or(false),
            process_state: world.process.value.clone(),
        }
    }
}

impl ProcessActivity {
    fn from_process_state(process: &ProcessState, expected_kinds: &[ActiveJobKind]) -> Self {
        match process {
            ProcessState::Running { active, .. } => {
                if expected_kinds.contains(&active.kind) {
                    Self::Running
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleSuccess
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome:
                    Some(JobOutcome::Failure { job_kind, .. } | JobOutcome::Timeout { job_kind, .. }),
                ..
            } => {
                if expected_kinds.contains(job_kind) {
                    Self::IdleFailure
                } else {
                    Self::IdleNoOutcome
                }
            }
            ProcessState::Idle {
                last_outcome: None, ..
            } => Self::IdleNoOutcome,
        }
    }
}

impl DecisionFacts {
    pub(crate) fn start_postgres_can_be_requested(&self) -> bool {
        !matches!(self.process_state, ProcessState::Running { .. })
    }

    pub(crate) fn rewind_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::PgRewind])
    }

    pub(crate) fn bootstrap_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(
            &self.process_state,
            &[ActiveJobKind::BaseBackup, ActiveJobKind::Bootstrap],
        )
    }

    pub(crate) fn fencing_activity(&self) -> ProcessActivity {
        ProcessActivity::from_process_state(&self.process_state, &[ActiveJobKind::Fencing])
    }
}

impl PhaseOutcome {
    pub(crate) fn new(next_phase: HaPhase, decision: HaDecision) -> Self {
        Self {
            next_phase,
            decision,
        }
    }
}

impl HaDecision {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::NoChange => "no_change",
            Self::WaitForPostgres { .. } => "wait_for_postgres",
            Self::WaitForDcsTrust => "wait_for_dcs_trust",
            Self::AttemptLeadership => "attempt_leadership",
            Self::FollowLeader { .. } => "follow_leader",
            Self::BecomePrimary { .. } => "become_primary",
            Self::StepDown(_) => "step_down",
            Self::RecoverReplica { .. } => "recover_replica",
            Self::FenceNode => "fence_node",
            Self::ReleaseLeaderLease { .. } => "release_leader_lease",
            Self::EnterFailSafe { .. } => "enter_fail_safe",
        }
    }

    pub(crate) fn detail(&self) -> Option<String> {
        match self {
            Self::NoChange | Self::WaitForDcsTrust | Self::AttemptLeadership | Self::FenceNode => {
                None
            }
            Self::WaitForPostgres {
                start_requested,
                leader_member_id,
            } => {
                let leader_detail = leader_member_id
                    .as_ref()
                    .map(|leader| leader.0.as_str())
                    .unwrap_or("none");
                Some(format!(
                    "start_requested={start_requested}, leader_member_id={leader_detail}"
                ))
            }
            Self::FollowLeader { leader_member_id } => Some(leader_member_id.0.clone()),
            Self::BecomePrimary { promote } => Some(format!("promote={promote}")),
            Self::StepDown(plan) => Some(format!(
                "reason={}, release_leader_lease={}, clear_switchover={}, fence={}",
                plan.reason.label(),
                plan.release_leader_lease,
                plan.clear_switchover,
                plan.fence
            )),
            Self::RecoverReplica { strategy } => Some(strategy.label()),
            Self::ReleaseLeaderLease { reason } => Some(reason.label()),
            Self::EnterFailSafe {
                release_leader_lease,
            } => Some(format!("release_leader_lease={release_leader_lease}")),
        }
    }
}

impl StepDownReason {
    fn label(&self) -> String {
        match self {
            Self::Switchover => "switchover".to_string(),
            Self::ForeignLeaderDetected { leader_member_id } => {
                format!("foreign_leader_detected:{}", leader_member_id.0)
            }
        }
    }
}

impl RecoveryStrategy {
    fn label(&self) -> String {
        match self {
            Self::Rewind { leader_member_id } => format!("rewind:{}", leader_member_id.0),
            Self::BaseBackup { leader_member_id } => {
                format!("base_backup:{}", leader_member_id.0)
            }
            Self::Bootstrap => "bootstrap".to_string(),
        }
    }
}

impl LeaseReleaseReason {
    fn label(&self) -> String {
        match self {
            Self::FencingComplete => "fencing_complete".to_string(),
            Self::PostgresUnreachable => "postgres_unreachable".to_string(),
        }
    }
}

fn is_postgres_reachable(state: &PgInfoState) -> bool {
    let sql = match state {
        PgInfoState::Unknown { common } => &common.sql,
        PgInfoState::Primary { common, .. } => &common.sql,
        PgInfoState::Replica { common, .. } => &common.sql,
    };
    matches!(sql, SqlStatus::Healthy)
}

fn is_local_primary(state: &PgInfoState) -> bool {
    matches!(
        state,
        PgInfoState::Primary {
            common,
            ..
        } if matches!(common.sql, SqlStatus::Healthy)
    )
}

fn should_rewind_from_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let Some(local_timeline) = pg_timeline(&world.pg.value) else {
        return false;
    };

    let leader_timeline = world
        .dcs
        .value
        .cache
        .members
        .get(leader_member_id)
        .and_then(|member| member.timeline);

    leader_timeline
        .map(|timeline| timeline != local_timeline)
        .unwrap_or(false)
}

fn pg_timeline(state: &PgInfoState) -> Option<TimelineId> {
    match state {
        PgInfoState::Unknown { common } => common.timeline,
        PgInfoState::Primary { common, .. } => common.timeline,
        PgInfoState::Replica { common, .. } => common.timeline,
    }
}

fn is_available_primary_leader(world: &WorldSnapshot, leader_member_id: &MemberId) -> bool {
    let leader_record = world.dcs.value.cache.members.get(leader_member_id);

    let Some(member) = leader_record else {
        // Preserve current behavior when leader member metadata is not yet observed.
        return true;
    };

    matches!(member.role, crate::dcs::state::MemberRole::Primary)
        && matches!(member.sql, SqlStatus::Healthy)
}


===== src/test_harness/ha_e2e/startup.rs =====
use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::config::{
    BinaryPaths, DcsConfig, DcsInitConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig,
    LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConfig,
    PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig,
    ProcessConfig, RoleAuthConfig, SecretSource, StderrSinkConfig,
};
use crate::state::WorkerError;
use crate::test_harness::binaries::{
    require_etcd_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
};
use crate::test_harness::etcd3::{
    prepare_etcd_member_data_dir, spawn_etcd3_cluster, EtcdClusterHandle, EtcdClusterMemberSpec,
    EtcdClusterSpec,
};
use crate::test_harness::namespace::NamespaceGuard;
use crate::test_harness::net_proxy::TcpProxyLink;
use crate::test_harness::pg16::prepare_pgdata_dir;
use crate::test_harness::ports::{allocate_ha_topology_ports, PortReservation};

use super::config::{Mode, TestConfig};
use super::handle::{NodeHandle, TestClusterHandle};
use super::util::{
    parse_http_endpoint, parse_loopback_socket, reserve_non_overlapping_ports,
    wait_for_bootstrap_primary, wait_for_node_api_ready_or_task_exit,
};

const ETCD_CLUSTER_STARTUP_TIMEOUT: Duration = Duration::from_secs(15);
const HARNESS_POSTGRES_CONNECT_TIMEOUT_S: u32 = 2;
const HARNESS_HA_LOOP_INTERVAL_MS: u64 = 100;
const HARNESS_HA_LEASE_TTL_MS: u64 = 2_000;
const HARNESS_PG_REWIND_TIMEOUT_MS: u64 = 5_000;
const HARNESS_BOOTSTRAP_TIMEOUT_MS: u64 = 30_000;
const HARNESS_FENCING_TIMEOUT_MS: u64 = 5_000;
const HARNESS_LOGGING_POLL_INTERVAL_MS: u64 = 200;
const HARNESS_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const HARNESS_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const HARNESS_LOGGING_PROTECT_RECENT_SECONDS: u64 = 300;

fn inline_secret(content: &str) -> SecretSource {
    SecretSource(InlineOrPath::Inline {
        content: content.to_string(),
    })
}

fn password_auth(content: &str) -> RoleAuthConfig {
    RoleAuthConfig::Password {
        password: inline_secret(content),
    }
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

struct StartupGuard {
    guard: NamespaceGuard,
    binaries: BinaryPaths,
    superuser_username: Option<String>,
    superuser_dbname: Option<String>,
    etcd: Option<EtcdClusterHandle>,
    nodes: Vec<NodeHandle>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    etcd_proxies: BTreeMap<String, TcpProxyLink>,
    api_proxies: BTreeMap<String, TcpProxyLink>,
    pg_proxies: BTreeMap<String, TcpProxyLink>,
    timeouts: super::config::TimeoutConfig,
}

impl StartupGuard {
    async fn cleanup_best_effort(&mut self) -> Result<(), WorkerError> {
        let mut failures = Vec::new();

        for task in &self.tasks {
            task.abort();
        }
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
        }

        for node in &self.nodes {
            if let Err(err) = super::util::pg_ctl_stop_immediate(
                self.binaries.pg_ctl.as_path(),
                node.data_dir.as_path(),
                self.timeouts.command_timeout,
                self.timeouts.command_kill_wait_timeout,
            )
            .await
            {
                failures.push(format!("postgres stop {} failed: {err}", node.id));
            }
        }

        let etcd_proxy_map = std::mem::take(&mut self.etcd_proxies);
        for (name, proxy) in etcd_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("etcd proxy {name} shutdown failed: {err}"));
            }
        }

        let api_proxy_map = std::mem::take(&mut self.api_proxies);
        for (name, proxy) in api_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("api proxy {name} shutdown failed: {err}"));
            }
        }

        let pg_proxy_map = std::mem::take(&mut self.pg_proxies);
        for (name, proxy) in pg_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("postgres proxy {name} shutdown failed: {err}"));
            }
        }

        if let Some(etcd) = self.etcd.as_mut() {
            if let Err(err) = etcd.shutdown_all().await {
                failures.push(format!("etcd shutdown failed: {err}"));
            }
        }
        self.etcd = None;

        if failures.is_empty() {
            Ok(())
        } else {
            Err(WorkerError::Message(format!(
                "startup cleanup failures: {}",
                failures.join("; ")
            )))
        }
    }

    fn into_handle(self) -> Result<TestClusterHandle, WorkerError> {
        let superuser_username = self.superuser_username.ok_or_else(|| {
            WorkerError::Message("startup missing postgres superuser username".to_string())
        })?;
        let superuser_dbname = self.superuser_dbname.ok_or_else(|| {
            WorkerError::Message("startup missing postgres superuser dbname".to_string())
        })?;

        Ok(TestClusterHandle {
            guard: self.guard,
            timeouts: self.timeouts,
            binaries: self.binaries,
            superuser_username,
            superuser_dbname,
            etcd: self.etcd,
            nodes: self.nodes,
            tasks: self.tasks,
            etcd_proxies: self.etcd_proxies,
            api_proxies: self.api_proxies,
            pg_proxies: self.pg_proxies,
        })
    }
}

pub async fn start_cluster(config: TestConfig) -> Result<TestClusterHandle, WorkerError> {
    let mut config = config;
    config.validate()?;

    let namespace_guard = NamespaceGuard::new(config.test_name.as_str())?;
    let namespace_id = namespace_guard.namespace()?.id.clone();
    config.scope = format!("{}-{}", config.scope, namespace_id);
    config.cluster_name = format!("{}-{}", config.cluster_name, namespace_id);

    let binaries = require_pg16_process_binaries_for_real_tests()?;
    let etcd_bin = require_etcd_bin_for_real_tests()?;

    let mut guard = StartupGuard {
        guard: namespace_guard,
        binaries: binaries.clone(),
        superuser_username: None,
        superuser_dbname: None,
        etcd: None,
        nodes: Vec::new(),
        tasks: Vec::new(),
        etcd_proxies: BTreeMap::new(),
        api_proxies: BTreeMap::new(),
        pg_proxies: BTreeMap::new(),
        timeouts: config.timeouts.clone(),
    };

    match start_cluster_inner(&mut guard, config, etcd_bin, binaries).await {
        Ok(()) => guard.into_handle(),
        Err(start_err) => {
            let cleanup_result = guard.cleanup_best_effort().await;
            match cleanup_result {
                Ok(()) => Err(start_err),
                Err(cleanup_err) => Err(WorkerError::Message(format!(
                    "{start_err}; cleanup failed: {cleanup_err}"
                ))),
            }
        }
    }
}

async fn start_cluster_inner(
    guard: &mut StartupGuard,
    config: TestConfig,
    etcd_bin: PathBuf,
    binaries: BinaryPaths,
) -> Result<(), WorkerError> {
    let namespace = guard.guard.namespace()?.clone();
    let etcd_member_count = config.etcd_members.len();
    let mut topology_reservation =
        allocate_ha_topology_ports(config.node_count, etcd_member_count)?;
    let topology = topology_reservation.layout().clone();
    let node_ports = topology.node_ports.clone();
    let postgres_roles = config.postgres_roles.clone().unwrap_or_default();

    let mut forbidden_ports: BTreeSet<u16> = topology
        .etcd_client_ports
        .iter()
        .chain(topology.etcd_peer_ports.iter())
        .chain(node_ports.iter())
        .copied()
        .collect();

    let mut members = Vec::with_capacity(etcd_member_count);
    for (index, member_name) in config.etcd_members.iter().enumerate() {
        let data_dir = prepare_etcd_member_data_dir(&namespace, member_name)?;
        let log_dir = namespace.child_dir(format!("logs/{member_name}"));
        let client_port = *topology.etcd_client_ports.get(index).ok_or_else(|| {
            WorkerError::Message(format!("missing etcd client port for {member_name}"))
        })?;
        let peer_port = *topology.etcd_peer_ports.get(index).ok_or_else(|| {
            WorkerError::Message(format!("missing etcd peer port for {member_name}"))
        })?;

        members.push(EtcdClusterMemberSpec {
            member_name: member_name.clone(),
            data_dir,
            log_dir,
            client_port,
            peer_port,
        });
    }

    let cluster_spec = EtcdClusterSpec {
        etcd_bin,
        namespace_id: namespace.id.clone(),
        startup_timeout: ETCD_CLUSTER_STARTUP_TIMEOUT,
        members,
    };

    for port in topology
        .etcd_client_ports
        .iter()
        .chain(topology.etcd_peer_ports.iter())
    {
        topology_reservation.release_port(*port).map_err(|err| {
            WorkerError::Message(format!("release etcd reserved port failed: {err}"))
        })?;
    }

    let etcd = spawn_etcd3_cluster(cluster_spec).await?;
    let endpoints = etcd.client_endpoints().to_vec();
    let endpoint_count = endpoints.len();
    if endpoint_count == 0 {
        return Err(WorkerError::Message(
            "etcd cluster returned no endpoints".to_string(),
        ));
    }
    guard.etcd = Some(etcd);

    let mut api_reservation = reserve_non_overlapping_ports(config.node_count, &forbidden_ports)?;
    let api_ports = api_reservation.as_slice().to_vec();
    if api_ports.len() != config.node_count {
        return Err(WorkerError::Message(format!(
            "api port reservation mismatch: expected {}, got {}",
            config.node_count,
            api_ports.len()
        )));
    }
    for port in &api_ports {
        forbidden_ports.insert(*port);
    }

    let mut cursor = 0usize;
    let mut proxy_reservation = PortReservation::empty();
    let (dcs_endpoints_by_node, proxy_ports) = match config.mode {
        Mode::Plain => (None, Vec::new()),
        Mode::PartitionProxy => {
            let total_proxy_ports = config.node_count.checked_mul(3).ok_or_else(|| {
                WorkerError::Message("proxy port count overflow for partition mode".to_string())
            })?;
            proxy_reservation = reserve_non_overlapping_ports(total_proxy_ports, &forbidden_ports)?;
            let proxy_ports = proxy_reservation.as_slice().to_vec();
            let dcs_endpoints_by_node = spawn_partition_etcd_proxies(
                guard,
                config.node_count,
                &endpoints,
                proxy_ports.as_slice(),
                &mut cursor,
                &mut proxy_reservation,
            )
            .await?;
            (Some(dcs_endpoints_by_node), proxy_ports)
        }
    };

    let next_proxy_listener = |ports: &[u16],
                               cursor_ref: &mut usize,
                               reservation: &mut PortReservation|
     -> Result<std::net::TcpListener, WorkerError> {
        if *cursor_ref >= ports.len() {
            return Err(WorkerError::Message(
                "proxy port allocation cursor out of bounds".to_string(),
            ));
        }
        let selected = ports[*cursor_ref];
        *cursor_ref = cursor_ref.saturating_add(1);
        reservation.take_listener(selected).map_err(|err| {
            WorkerError::Message(format!(
                "take proxy reserved listener failed for port={selected}: {err}"
            ))
        })
    };

    for (index, (pg_port, api_port)) in node_ports.iter().copied().zip(api_ports).enumerate() {
        let node_id = format!("node-{}", index.saturating_add(1));
        let data_dir = prepare_pgdata_dir(&namespace, &node_id)?;
        let socket_dir = namespace.child_dir(format!("run/{node_id}"));
        let log_file = namespace.child_dir(format!("logs/{node_id}/postgres.log"));
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                WorkerError::Message(format!(
                    "create postgres log dir failed for node {node_id}: {err}"
                ))
            })?;
        }

        let api_addr: SocketAddr = format!("127.0.0.1:{api_port}")
            .parse()
            .map_err(|err| WorkerError::Message(format!("parse api addr failed: {err}")))?;

        let (api_observe_addr, sql_port) = match config.mode {
            Mode::Plain => (api_addr, pg_port),
            Mode::PartitionProxy => {
                let api_listener = next_proxy_listener(
                    proxy_ports.as_slice(),
                    &mut cursor,
                    &mut proxy_reservation,
                )?;
                let api_proxy = TcpProxyLink::spawn_with_listener(
                    format!("{node_id}-api-proxy"),
                    api_listener,
                    api_addr,
                )
                .await
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "spawn api proxy failed for node {node_id}: {err}"
                    ))
                })?;
                let api_proxy_addr = api_proxy.listen_addr();
                guard.api_proxies.insert(node_id.clone(), api_proxy);

                let pg_listener = next_proxy_listener(
                    proxy_ports.as_slice(),
                    &mut cursor,
                    &mut proxy_reservation,
                )?;
                let pg_target_addr = parse_loopback_socket(pg_port)?;
                let pg_proxy = TcpProxyLink::spawn_with_listener(
                    format!("{node_id}-pg-proxy"),
                    pg_listener,
                    pg_target_addr,
                )
                .await
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "spawn postgres proxy failed for node {node_id}: {err}"
                    ))
                })?;
                let pg_proxy_addr = pg_proxy.listen_addr();
                guard.pg_proxies.insert(node_id.clone(), pg_proxy);

                (api_proxy_addr, pg_proxy_addr.port())
            }
        };

        let dcs_endpoints = match (config.mode, &dcs_endpoints_by_node) {
            (Mode::Plain, _) => endpoints.clone(),
            (Mode::PartitionProxy, Some(map)) => {
                map.get(node_id.as_str()).cloned().ok_or_else(|| {
                    WorkerError::Message(format!(
                        "missing proxy DCS endpoints for node runtime config: {node_id}"
                    ))
                })?
            }
            (Mode::PartitionProxy, None) => {
                return Err(WorkerError::Message(
                    "partition mode missing DCS endpoints map".to_string(),
                ));
            }
        };

        let replicator_username = postgres_roles.replicator_username.clone();
        let replicator_password = postgres_roles.replicator_password.clone();
        let rewinder_username = postgres_roles.rewinder_username.clone();
        let rewinder_password = postgres_roles.rewinder_password.clone();
        let pg_hba_contents = format!(
            concat!(
                "# managed by pgtuskmaster test harness\n",
                "local all all trust\n",
                "host replication {} 127.0.0.1/32 trust\n",
                "host all {} 127.0.0.1/32 trust\n",
                "host all all 127.0.0.1/32 trust\n",
            ),
            replicator_username, rewinder_username,
        );
        let pg_ident_contents = "# empty\n".to_string();

        let dcs_endpoints_for_check = dcs_endpoints.clone();
        let dcs_init_payload = serde_json::json!({
            "cluster": {
                "name": config.cluster_name.clone(),
                "member_id": node_id.clone(),
            },
            "postgres": {
                "data_dir": data_dir.display().to_string(),
                "connect_timeout_s": HARNESS_POSTGRES_CONNECT_TIMEOUT_S,
                "listen_host": "127.0.0.1",
                "listen_port": pg_port,
                "socket_dir": socket_dir.display().to_string(),
                "log_file": log_file.display().to_string(),
                "local_conn_identity": { "user": "postgres", "dbname": "postgres", "ssl_mode": "prefer" },
                "rewind_conn_identity": { "user": rewinder_username.clone(), "dbname": "postgres", "ssl_mode": "prefer" },
                "tls": { "mode": "disabled", "identity": null, "client_auth": null },
                "roles": {
                    "superuser": { "username": "postgres", "auth": { "type": "password", "password": { "content": "secret-password" } } },
                    "replicator": { "username": replicator_username.clone(), "auth": { "type": "password", "password": { "content": replicator_password.clone() } } },
                    "rewinder": { "username": rewinder_username.clone(), "auth": { "type": "password", "password": { "content": rewinder_password.clone() } } },
                },
                "pg_hba": { "source": { "content": pg_hba_contents.clone() } },
                "pg_ident": { "source": { "content": pg_ident_contents.clone() } },
                "extra_gucs": {},
            },
            "dcs": {
                "endpoints": dcs_endpoints_for_check.clone(),
                "scope": config.scope.clone(),
                "init": null,
            },
            "ha": {
                "loop_interval_ms": HARNESS_HA_LOOP_INTERVAL_MS,
                "lease_ttl_ms": HARNESS_HA_LEASE_TTL_MS,
            },
            "process": {
                "pg_rewind_timeout_ms": HARNESS_PG_REWIND_TIMEOUT_MS,
                "bootstrap_timeout_ms": HARNESS_BOOTSTRAP_TIMEOUT_MS,
                "fencing_timeout_ms": HARNESS_FENCING_TIMEOUT_MS,
                "binaries": {
                    "postgres": binaries.postgres.display().to_string(),
                    "pg_ctl": binaries.pg_ctl.display().to_string(),
                    "pg_rewind": binaries.pg_rewind.display().to_string(),
                    "initdb": binaries.initdb.display().to_string(),
                    "pg_basebackup": binaries.pg_basebackup.display().to_string(),
                    "psql": binaries.psql.display().to_string(),
                },
            },
            "logging": {
                "level": "info",
                "capture_subprocess_output": false,
                "postgres": {
                    "enabled": false,
                    "pg_ctl_log_file": null,
                    "log_dir": null,
                    "poll_interval_ms": HARNESS_LOGGING_POLL_INTERVAL_MS,
                    "cleanup": {
                        "enabled": true,
                        "max_files": HARNESS_LOGGING_CLEANUP_MAX_FILES,
                        "max_age_seconds": HARNESS_LOGGING_CLEANUP_MAX_AGE_SECONDS,
                        "protect_recent_seconds": HARNESS_LOGGING_PROTECT_RECENT_SECONDS
                    },
                },
                "sinks": {
                    "stderr": { "enabled": true },
                    "file": { "enabled": false, "path": null, "mode": "append" },
                },
            },
            "api": {
                "listen_addr": api_addr.to_string(),
                "security": {
                    "tls": { "mode": "disabled", "identity": null, "client_auth": null },
                    "auth": { "type": "disabled" },
                },
            },
            "debug": { "enabled": false },
        });
        let dcs_init_payload_json = serde_json::to_string(&dcs_init_payload).map_err(|err| {
            WorkerError::Message(format!(
                "encode dcs.init.payload_json failed for node {node_id}: {err}"
            ))
        })?;
        let runtime_replicator_username = replicator_username.clone();
        let runtime_replicator_password = replicator_password.clone();
        let runtime_rewinder_username = rewinder_username.clone();
        let runtime_rewinder_password = rewinder_password.clone();

        let runtime_cfg = crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_cluster_name(config.cluster_name.clone())
            .with_member_id(node_id.clone())
            .transform_postgres(|postgres| PostgresConfig {
                data_dir: data_dir.clone(),
                connect_timeout_s: HARNESS_POSTGRES_CONNECT_TIMEOUT_S,
                listen_port: pg_port,
                socket_dir,
                log_file: log_file.clone(),
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: runtime_rewinder_username.clone(),
                    ..postgres.rewind_conn_identity
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        auth: password_auth("secret-password"),
                        ..postgres.roles.superuser
                    },
                    replicator: PostgresRoleConfig {
                        username: runtime_replicator_username.clone(),
                        auth: password_auth(runtime_replicator_password.as_str()),
                    },
                    rewinder: PostgresRoleConfig {
                        username: runtime_rewinder_username.clone(),
                        auth: password_auth(runtime_rewinder_password.as_str()),
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: pg_hba_contents.clone(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: pg_ident_contents.clone(),
                    },
                },
                ..postgres
            })
            .with_dcs(DcsConfig {
                endpoints: dcs_endpoints,
                scope: config.scope.clone(),
                init: Some(DcsInitConfig {
                    payload_json: dcs_init_payload_json.clone(),
                    write_on_bootstrap: true,
                }),
            })
            .with_ha(HaConfig {
                loop_interval_ms: HARNESS_HA_LOOP_INTERVAL_MS,
                lease_ttl_ms: HARNESS_HA_LEASE_TTL_MS,
            })
            .with_process(ProcessConfig {
                pg_rewind_timeout_ms: HARNESS_PG_REWIND_TIMEOUT_MS,
                bootstrap_timeout_ms: HARNESS_BOOTSTRAP_TIMEOUT_MS,
                fencing_timeout_ms: HARNESS_FENCING_TIMEOUT_MS,
                binaries: binaries.clone(),
            })
            .with_logging(LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: false,
                postgres: PostgresLoggingConfig {
                    enabled: false,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: HARNESS_LOGGING_POLL_INTERVAL_MS,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: HARNESS_LOGGING_CLEANUP_MAX_FILES,
                        max_age_seconds: HARNESS_LOGGING_CLEANUP_MAX_AGE_SECONDS,
                        protect_recent_seconds: HARNESS_LOGGING_PROTECT_RECENT_SECONDS,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
                    },
                },
            })
            .with_api_listen_addr(api_addr.to_string())
            .with_debug(DebugConfig { enabled: false })
            .build();

        let runtime_superuser_username = runtime_cfg.postgres.roles.superuser.username.clone();
        let runtime_superuser_dbname = runtime_cfg.postgres.local_conn_identity.dbname.clone();
        match (&guard.superuser_username, &guard.superuser_dbname) {
            (None, None) => {
                guard.superuser_username = Some(runtime_superuser_username);
                guard.superuser_dbname = Some(runtime_superuser_dbname);
            }
            (Some(expected_user), Some(expected_dbname)) => {
                if expected_user.as_str() != runtime_superuser_username.as_str()
                    || expected_dbname.as_str() != runtime_superuser_dbname.as_str()
                {
                    return Err(WorkerError::Message(format!(
                        "inconsistent superuser identity across nodes: expected user/dbname {}/{} but got {}/{}",
                        expected_user,
                        expected_dbname,
                        runtime_superuser_username,
                        runtime_superuser_dbname
                    )));
                }
            }
            _ => {
                return Err(WorkerError::Message(
                    "startup guard superuser identity partially initialized".to_string(),
                ));
            }
        }

        topology_reservation.release_port(pg_port).map_err(|err| {
            WorkerError::Message(format!("release postgres reserved port failed: {err}"))
        })?;
        api_reservation.release_port(api_port).map_err(|err| {
            WorkerError::Message(format!("release api reserved port failed: {err}"))
        })?;

        let task_node_id = node_id.clone();
        let runtime_task = tokio::task::spawn_local(async move {
            match crate::runtime::run_node_from_config(runtime_cfg).await {
                Ok(()) => Ok(()),
                Err(err) => Err(WorkerError::Message(format!(
                    "runtime node {task_node_id} exited with error: {err}"
                ))),
            }
        });

        guard.nodes.push(NodeHandle {
            id: node_id.clone(),
            pg_port,
            sql_port,
            api_addr,
            api_observe_addr,
            data_dir,
        });

        let runtime_task = wait_for_node_api_ready_or_task_exit(
            api_observe_addr,
            node_id.as_str(),
            log_file.as_path(),
            runtime_task,
            config.timeouts.http_step_timeout,
            config.timeouts.api_readiness_timeout,
        )
        .await?;
        guard.tasks.push(runtime_task);

        if index == 0 {
            let expected_member_id = format!("node-{}", index.saturating_add(1));
            wait_for_bootstrap_primary(
                api_observe_addr,
                expected_member_id.as_str(),
                config.timeouts.http_step_timeout,
                config.timeouts.bootstrap_primary_timeout,
            )
            .await?;

            // Clone/basebackup connects using the configured replicator role. Ensure that role
            // exists on the elected primary before bringing up other nodes.
            let superuser_username = guard.superuser_username.as_deref().ok_or_else(|| {
                WorkerError::Message("startup missing postgres superuser username".to_string())
            })?;
            let superuser_dbname = guard.superuser_dbname.as_deref().ok_or_else(|| {
                WorkerError::Message("startup missing postgres superuser dbname".to_string())
            })?;

            let create_roles_sql = r#"
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {replicator_name}) THEN
    EXECUTE format(
      'CREATE ROLE %I WITH LOGIN REPLICATION PASSWORD %L',
      {replicator_name},
      {replicator_password}
    );
  ELSE
    EXECUTE format(
      'ALTER ROLE %I WITH LOGIN REPLICATION PASSWORD %L',
      {replicator_name},
      {replicator_password}
    );
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {rewinder_name}) THEN
    -- pg_rewind typically needs superuser privileges; keep tests conservative.
    EXECUTE format(
      'CREATE ROLE %I WITH LOGIN SUPERUSER PASSWORD %L',
      {rewinder_name},
      {rewinder_password}
    );
  ELSE
    EXECUTE format(
      'ALTER ROLE %I WITH LOGIN SUPERUSER PASSWORD %L',
      {rewinder_name},
      {rewinder_password}
    );
  END IF;
END
$$;
"#;
            let create_roles_sql = create_roles_sql
                .replace(
                    "{replicator_name}",
                    sql_literal(replicator_username.as_str()).as_str(),
                )
                .replace(
                    "{replicator_password}",
                    sql_literal(replicator_password.as_str()).as_str(),
                )
                .replace(
                    "{rewinder_name}",
                    sql_literal(rewinder_username.as_str()).as_str(),
                )
                .replace(
                    "{rewinder_password}",
                    sql_literal(rewinder_password.as_str()).as_str(),
                );
            let _ = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                create_roles_sql.as_str(),
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;

            let primary = guard.nodes.last().ok_or_else(|| {
                WorkerError::Message("startup expected primary node handle".to_string())
            })?;
            let expected_hba_file = primary.data_dir.join("pgtm.pg_hba.conf");
            let expected_ident_file = primary.data_dir.join("pgtm.pg_ident.conf");
            let expected_managed_postgresql_conf = primary.data_dir.join("pgtm.postgresql.conf");

            let hba_file_raw = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                "SHOW hba_file;",
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;
            let ident_file_raw = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                "SHOW ident_file;",
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;
            let config_file_raw = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                "SHOW config_file;",
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;

            let expected_hba = expected_hba_file.display().to_string();
            let expected_ident = expected_ident_file.display().to_string();
            let expected_config = expected_managed_postgresql_conf.display().to_string();
            if hba_file_raw.trim() != expected_hba.as_str() {
                return Err(WorkerError::Message(format!(
                    "expected SHOW hba_file to be `{expected_hba}`, got: {:?}",
                    hba_file_raw.trim()
                )));
            }
            if ident_file_raw.trim() != expected_ident.as_str() {
                return Err(WorkerError::Message(format!(
                    "expected SHOW ident_file to be `{expected_ident}`, got: {:?}",
                    ident_file_raw.trim()
                )));
            }
            if config_file_raw.trim() != expected_config.as_str() {
                return Err(WorkerError::Message(format!(
                    "expected SHOW config_file to be `{expected_config}`, got: {:?}",
                    config_file_raw.trim()
                )));
            }

            let disk_hba = std::fs::read_to_string(&expected_hba_file).map_err(|err| {
                WorkerError::Message(format!(
                    "read managed hba file {} failed: {err}",
                    expected_hba_file.display()
                ))
            })?;
            if disk_hba != pg_hba_contents {
                return Err(WorkerError::Message(format!(
                    "managed hba file did not match configured content; file={} expected_len={} actual_len={}",
                    expected_hba_file.display(),
                    pg_hba_contents.len(),
                    disk_hba.len(),
                )));
            }
            let disk_ident = std::fs::read_to_string(&expected_ident_file).map_err(|err| {
                WorkerError::Message(format!(
                    "read managed ident file {} failed: {err}",
                    expected_ident_file.display()
                ))
            })?;
            if disk_ident != pg_ident_contents {
                return Err(WorkerError::Message(format!(
                    "managed ident file did not match configured content; file={} expected_len={} actual_len={}",
                    expected_ident_file.display(),
                    pg_ident_contents.len(),
                    disk_ident.len(),
                )));
            }
            let disk_managed_postgresql_conf =
                std::fs::read_to_string(&expected_managed_postgresql_conf).map_err(|err| {
                    WorkerError::Message(format!(
                        "read managed postgresql conf {} failed: {err}",
                        expected_managed_postgresql_conf.display()
                    ))
                })?;
            if disk_managed_postgresql_conf.contains("archive_mode")
                || disk_managed_postgresql_conf.contains("archive_command")
                || disk_managed_postgresql_conf.contains("restore_command")
            {
                return Err(WorkerError::Message(format!(
                    "managed postgresql conf unexpectedly contains backup settings: {:?}",
                    disk_managed_postgresql_conf
                )));
            }
            if !disk_managed_postgresql_conf.contains(expected_hba.as_str())
                || !disk_managed_postgresql_conf.contains(expected_ident.as_str())
            {
                return Err(WorkerError::Message(format!(
                    "managed postgresql conf did not reference expected managed hba/ident files: {:?}",
                    disk_managed_postgresql_conf
                )));
            }
            if !disk_managed_postgresql_conf.contains("listen_addresses = '127.0.0.1'")
                || !disk_managed_postgresql_conf.contains(format!("port = {pg_port}").as_str())
            {
                return Err(WorkerError::Message(format!(
                    "managed postgresql conf missing expected listen/port settings: {:?}",
                    disk_managed_postgresql_conf
                )));
            }

            let init_key = format!("/{}/init", config.scope.trim_matches('/'));
            let config_key = format!("/{}/config", config.scope.trim_matches('/'));
            let mut etcd_client =
                etcd_client::Client::connect(dcs_endpoints_for_check.clone(), None)
                    .await
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "etcd connect for init/config check failed: {err}"
                        ))
                    })?;
            let init_response = etcd_client
                .get(init_key.as_str(), None)
                .await
                .map_err(|err| WorkerError::Message(format!("etcd get init key failed: {err}")))?;
            if init_response.kvs().is_empty() {
                return Err(WorkerError::Message(format!(
                    "expected init key to exist at {init_key}"
                )));
            }

            let config_response =
                etcd_client
                    .get(config_key.as_str(), None)
                    .await
                    .map_err(|err| {
                        WorkerError::Message(format!("etcd get config key failed: {err}"))
                    })?;
            let Some(kv) = config_response.kvs().first() else {
                return Err(WorkerError::Message(format!(
                    "expected config key to exist at {config_key}"
                )));
            };
            let raw = std::str::from_utf8(kv.value()).map_err(|err| {
                WorkerError::Message(format!("config value not utf8 at {config_key}: {err}"))
            })?;
            let decoded: serde_json::Value = serde_json::from_str(raw).map_err(|err| {
                WorkerError::Message(format!(
                    "config payload stored in etcd was not valid json at {config_key}: {err}"
                ))
            })?;
            if decoded != dcs_init_payload {
                return Err(WorkerError::Message(format!(
                    "etcd config payload mismatch: expected={dcs_init_payload_json} got={raw}"
                )));
            }
        }
    }

    if config.mode == Mode::PartitionProxy && cursor != proxy_ports.len() {
        return Err(WorkerError::Message(format!(
            "proxy port cursor mismatch: used={cursor} allocated={}",
            proxy_ports.len()
        )));
    }

    // Keep port reservations alive until the entire cluster is ready; the runtime binds
    // ports asynchronously after we release the OS-level reservation sockets.
    drop(proxy_reservation);
    drop(api_reservation);
    drop(topology_reservation);

    Ok(())
}

async fn spawn_partition_etcd_proxies(
    guard: &mut StartupGuard,
    node_count: usize,
    endpoints: &[String],
    proxy_ports: &[u16],
    cursor: &mut usize,
    proxy_reservation: &mut PortReservation,
) -> Result<BTreeMap<String, Vec<String>>, WorkerError> {
    let member_names = guard
        .etcd
        .as_ref()
        .ok_or_else(|| WorkerError::Message("missing etcd cluster handle".to_string()))?
        .member_names();
    if member_names.len() != endpoints.len() {
        return Err(WorkerError::Message(format!(
            "etcd members/endpoints mismatch: members={} endpoints={}",
            member_names.len(),
            endpoints.len()
        )));
    }

    let next_listener = |ports: &[u16],
                         cursor_ref: &mut usize,
                         reservation: &mut PortReservation|
     -> Result<std::net::TcpListener, WorkerError> {
        if *cursor_ref >= ports.len() {
            return Err(WorkerError::Message(
                "proxy port allocation cursor out of bounds".to_string(),
            ));
        }
        let selected = ports[*cursor_ref];
        *cursor_ref = cursor_ref.saturating_add(1);
        reservation.take_listener(selected).map_err(|err| {
            WorkerError::Message(format!(
                "take proxy reserved listener failed for port={selected}: {err}"
            ))
        })
    };

    let mut dcs_endpoints_by_node: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node_index in 0..node_count {
        let node_id = format!("node-{}", node_index.saturating_add(1));
        let endpoint_index = node_index % endpoints.len();
        let member_name = member_names.get(endpoint_index).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing etcd member name for endpoint index {endpoint_index}"
            ))
        })?;
        let endpoint = endpoints.get(endpoint_index).ok_or_else(|| {
            WorkerError::Message(format!("missing etcd endpoint for index {endpoint_index}"))
        })?;
        let target_addr = parse_http_endpoint(endpoint.as_str())?;
        let link_name = format!("{node_id}-to-{member_name}-etcd");
        let listener = next_listener(proxy_ports, cursor, proxy_reservation)?;
        let link = TcpProxyLink::spawn_with_listener(link_name.clone(), listener, target_addr)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "spawn etcd proxy failed for node {node_id} link={link_name}: {err}"
                ))
            })?;

        let proxy_url = format!("http://{}", link.listen_addr());
        guard.etcd_proxies.insert(link_name.clone(), link);
        dcs_endpoints_by_node.insert(node_id, vec![proxy_url]);
    }

    Ok(dcs_endpoints_by_node)
}


===== src/test_harness/ha_e2e/ops.rs =====
use crate::state::WorkerError;

use super::handle::TestClusterHandle;

impl TestClusterHandle {
    pub async fn shutdown(&mut self) -> Result<(), WorkerError> {
        let mut failures = Vec::new();

        for task in &self.tasks {
            task.abort();
        }
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
        }

        for node in &self.nodes {
            if let Err(err) = super::util::pg_ctl_stop_immediate(
                self.binaries.pg_ctl.as_path(),
                node.data_dir.as_path(),
                self.timeouts.command_timeout,
                self.timeouts.command_kill_wait_timeout,
            )
            .await
            {
                failures.push(format!("postgres stop {} failed: {err}", node.id));
            }
        }

        let etcd_proxy_map = std::mem::take(&mut self.etcd_proxies);
        for (name, proxy) in etcd_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("etcd proxy {name} shutdown failed: {err}"));
            }
        }

        let api_proxy_map = std::mem::take(&mut self.api_proxies);
        for (name, proxy) in api_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("api proxy {name} shutdown failed: {err}"));
            }
        }

        let pg_proxy_map = std::mem::take(&mut self.pg_proxies);
        for (name, proxy) in pg_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("postgres proxy {name} shutdown failed: {err}"));
            }
        }

        if let Some(etcd) = self.etcd.as_mut() {
            if let Err(err) = etcd.shutdown_all().await {
                failures.push(format!("etcd shutdown failed: {err}"));
            }
        }
        self.etcd = None;

        if failures.is_empty() {
            Ok(())
        } else {
            Err(WorkerError::Message(format!(
                "cluster shutdown failures: {}",
                failures.join("; ")
            )))
        }
    }
}


===== src/dcs/state.rs =====
use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    state::{
        MemberId, StatePublisher, StateSubscriber, TimelineId, UnixMillis, Version, WalLsn,
        WorkerStatus,
    },
};

use super::store::DcsStore;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DcsTrust {
    FullQuorum,
    FailSafe,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MemberRole {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) member_id: MemberId,
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) role: MemberRole,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) write_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) updated_at: UnixMillis,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderRecord {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SwitchoverRequest {
    pub(crate) requested_by: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InitLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCache {
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader: Option<LeaderRecord>,
    pub(crate) switchover: Option<SwitchoverRequest>,
    pub(crate) config: RuntimeConfig,
    pub(crate) init_lock: Option<InitLockRecord>,
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
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsState>,
    pub(crate) store: Box<dyn DcsStore>,
    pub(crate) log: LogHandle,
    pub(crate) cache: DcsCache,
    pub(crate) last_published_pg_version: Option<Version>,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
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
        return DcsTrust::FailSafe;
    };
    if !member_record_is_fresh(self_member, cache, now) {
        return DcsTrust::FailSafe;
    }

    if let Some(leader) = &cache.leader {
        let Some(leader_member) = cache.members.get(&leader.member_id) else {
            return DcsTrust::FailSafe;
        };
        if !member_record_is_fresh(leader_member, cache, now) {
            return DcsTrust::FailSafe;
        }
    }

    if cache.members.len() > 1 && fresh_member_count(cache, now) < 2 {
        return DcsTrust::FailSafe;
    }

    DcsTrust::FullQuorum
}

fn member_record_is_fresh(record: &MemberRecord, cache: &DcsCache, now: UnixMillis) -> bool {
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

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_host: &str,
    postgres_port: u16,
    pg_state: &PgInfoState,
    now: UnixMillis,
    pg_version: Version,
) -> MemberRecord {
    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: Some(*wal_lsn),
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: Some(*replay_lsn),
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
        build_local_member_record, evaluate_trust, DcsCache, DcsTrust, LeaderRecord, MemberRecord,
        MemberRole,
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
            init_lock: None,
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
            DcsTrust::FailSafe
        );

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                role: MemberRole::Unknown,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FullQuorum
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::FailSafe
        );

        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FailSafe
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
        let unknown_record = build_local_member_record(
            &self_id,
            "10.0.0.11",
            5433,
            &unknown,
            UnixMillis(10),
            Version(11),
        );
        assert_eq!(unknown_record.postgres_host, "10.0.0.11".to_string());
        assert_eq!(unknown_record.postgres_port, 5433);
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
        let primary_record = build_local_member_record(
            &self_id,
            "10.0.0.12",
            5434,
            &primary,
            UnixMillis(12),
            Version(13),
        );
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
        let replica_record = build_local_member_record(
            &self_id,
            "10.0.0.13",
            5435,
            &replica,
            UnixMillis(14),
            Version(15),
        );
        assert_eq!(replica_record.postgres_host, "10.0.0.13".to_string());
        assert_eq!(replica_record.postgres_port, 5435);
        assert_eq!(replica_record.role, MemberRole::Replica);
        assert_eq!(replica_record.write_lsn, None);
        assert_eq!(replica_record.replay_lsn, Some(WalLsn(22)));
    }
}


===== docs/tmp/verbose_extra_context/observing-failover-deep-summary.md =====
# Observing failover deep summary

This support note is only raw factual context for `docs/src/tutorial/observing-failover.md`.
Do not claim runtime commands, signals, or HTTP payloads unless the allowed files support them directly.

Published environment and cluster shape:

- `docker/compose/docker-compose.cluster.yml` defines one `etcd` service and three pgtuskmaster nodes.
- Each node exposes API port `8080` and PostgreSQL port `5432`.
- The compose setup gives each node its own persistent data and log volumes.
- This is a real multi-node topology, not a mock.

What the E2E suite proves exists:

- `tests/ha_multi_node_failover.rs` includes real HA scenarios such as:
  - unassisted failover with SQL consistency
  - stress/unassisted failover with concurrent SQL
  - no-quorum and fail-safe coverage
- `tests/ha_partition_isolation.rs` includes:
  - minority isolation without split brain
  - primary isolation with failover and no split brain
  - API-path isolation preserving primary
  - mixed-fault healing convergence
- For tutorial purposes, the safe claim is that the repo already tests both ordinary failover and network/isolation scenarios.

What learners can observe from the debug subsystem:

- `src/debug_api/snapshot.rs` defines `SystemSnapshot` with these domains:
  - app lifecycle
  - versioned `config`
  - versioned `pg`
  - versioned `dcs`
  - versioned `process`
  - versioned `ha`
  - `generated_at`
  - monotonic `sequence`
  - `changes`
  - `timeline`
- `src/debug_api/mod.rs` shows the debug API is built from `snapshot`, `view`, and `worker` modules.
- `src/debug_api/view.rs` builds a verbose payload that includes sections for config, pginfo, dcs, process, ha, api, debug, changes, and timeline.

DCS observation terms the tutorial can use:

- `DcsState` contains `worker`, `trust`, `cache`, and `last_refresh_at`.
- `DcsTrust` values are `FullQuorum`, `FailSafe`, and `NotTrusted`.
- The member cache carries per-member:
  - role
  - SQL health
  - readiness
  - timeline
  - WAL positions
  - update timestamp
- The cache also carries leader and switchover records.
- Trust evaluation becomes:
  - `NotTrusted` if etcd is unhealthy
  - `FailSafe` if self member is missing/stale
  - `FailSafe` if leader is missing/stale
  - `FailSafe` if multi-member freshness drops below two fresh records
  - `FullQuorum` otherwise

HA decisions and detail strings that are source-backed:

- `src/ha/decision.rs` serializes these decision kinds:
  - `no_change`
  - `wait_for_postgres`
  - `wait_for_dcs_trust`
  - `attempt_leadership`
  - `follow_leader`
  - `become_primary`
  - `step_down`
  - `recover_replica`
  - `fence_node`
  - `release_leader_lease`
  - `enter_fail_safe`
- Decision detail data that is safe to mention:
  - `wait_for_postgres { start_requested, leader_member_id }`
  - `become_primary { promote }`
  - `step_down { reason, release_leader_lease, clear_switchover, fence }`
  - `recover_replica { strategy = rewind | base_backup | bootstrap }`
  - `release_leader_lease { reason = fencing_complete | postgres_unreachable }`
  - `enter_fail_safe { release_leader_lease }`
- The HA layer also tracks job activity classes for rewind, bootstrap/base-backup, and fencing as:
  - `Running`
  - `IdleNoOutcome`
  - `IdleSuccess`
  - `IdleFailure`

Harness facts that explain why failover is observable quickly:

- `src/test_harness/ha_e2e/startup.rs` sets:
  - `loop_interval_ms = 100`
  - `lease_ttl_ms = 2000`
  - explicit timeouts for rewind, bootstrap, and fencing
- The startup harness waits for the first node to become the bootstrap primary before the rest of the cluster is brought up.
- It also verifies that DCS bootstrap keys exist under `/{scope}/init` and `/{scope}/config` before proceeding.

Source-backed answer to the extra question about fault injection:

- The exact per-test command lines or Unix signals are not fully visible in the allowed files.
- What is directly visible:
  - task teardown aborts runtime tasks with `task.abort()`
  - PostgreSQL shutdown uses the helper `pg_ctl_stop_immediate(...)`
  - partition mode inserts `TcpProxyLink` proxies in front of etcd, API, and PostgreSQL endpoints
- Safe wording:
  - connectivity-fault scenarios are simulated through network proxies
  - process/cluster teardown uses task abort and immediate PostgreSQL stop helpers
- Unsafe wording to avoid unless another source proves it:
  - do not say specific Unix signals are used
  - do not say the exact `pg_ctl` command-line flags unless supported by a lower-level helper file
