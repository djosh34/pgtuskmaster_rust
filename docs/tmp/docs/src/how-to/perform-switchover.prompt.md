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

docs/src/how-to/perform-switchover.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/how-to/check-cluster-health.md
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

# Explanation

- [Explanation]()
    - [Architecture](explanation/architecture.md)

# Reference

- [Reference]()
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
docs/draft/docs/src/how-to/check-cluster-health.md
docs/draft/docs/src/how-to/check-cluster-health.revised.md
docs/draft/docs/src/how-to/perform-switchover.md
docs/draft/docs/src/reference/cli-commands.md
docs/draft/docs/src/reference/cli-commands.revised.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.revised.md
docs/draft/docs/src/reference/cli.md
docs/draft/docs/src/reference/cli.revised.md
docs/draft/docs/src/reference/pgtuskmaster-cli.md
docs/draft/docs/src/reference/pgtuskmasterctl-cli.md
docs/draft/docs/src/reference/pgtuskmasterctl-cli.revised.md
docs/draft/docs/src/reference/runtime-configuration.md
docs/draft/docs/src/reference/runtime-configuration.revised.md
docs/draft/docs/src/tutorial/first-ha-cluster.final.md
docs/draft/docs/src/tutorial/first-ha-cluster.md
docs/draft/docs/src/tutorial/first-ha-cluster.revised.md
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/src/explanation/architecture.md
docs/src/how-to/check-cluster-health.md
docs/src/reference/pgtuskmasterctl-cli.md
docs/src/reference/runtime-configuration.md
docs/src/tutorial/first-ha-cluster.md
docs/tmp/docs/src/explanation/architecture.prompt.md
docs/tmp/docs/src/how-to/check-cluster-health.prompt.md
docs/tmp/docs/src/how-to/perform-switchover.prompt.md
docs/tmp/docs/src/reference/cli-commands.prompt.md
docs/tmp/docs/src/reference/cli-pgtuskmasterctl.prompt.md
docs/tmp/docs/src/reference/cli.prompt.md
docs/tmp/docs/src/reference/pgtuskmaster-cli.prompt.md
docs/tmp/docs/src/reference/pgtuskmasterctl-cli.prompt.md
docs/tmp/docs/src/reference/runtime-configuration.prompt.md
docs/tmp/docs/src/tutorial/first-ha-cluster.prompt.md
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
docs/tmp/verbose_extra_context/check-cluster-health-api-and-state.md
docs/tmp/verbose_extra_context/check-cluster-health-cli-overview.md
docs/tmp/verbose_extra_context/check-cluster-health-runtime-evidence.md
docs/tmp/verbose_extra_context/cli-surface-summary.md
docs/tmp/verbose_extra_context/cluster-start-command.md
docs/tmp/verbose_extra_context/leader-check-command.md
docs/tmp/verbose_extra_context/perform-switchover-deep-summary.md
docs/tmp/verbose_extra_context/pgtuskmaster-cli-deep-summary.md
docs/tmp/verbose_extra_context/runtime-config-deep-summary.md
docs/tmp/verbose_extra_context/runtime-config-summary.md


===== src/api/controller.rs =====
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        AcceptedResponse, ApiError, ApiResult, DcsTrustResponse, HaDecisionResponse,
        HaPhaseResponse, HaStateResponse, LeaseReleaseReasonResponse, RecoveryStrategyResponse,
        StepDownReasonResponse,
    },
    dcs::{
        state::{DcsTrust, SwitchoverRequest},
        store::{DcsHaWriter, DcsStore},
    },
    debug_api::snapshot::SystemSnapshot,
    ha::{
        decision::{
            HaDecision, LeaseReleaseReason, RecoveryStrategy, StepDownPlan, StepDownReason,
        },
        state::HaPhase,
    },
    state::{MemberId, Versioned},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequestInput {
    pub(crate) requested_by: MemberId,
}

pub(crate) fn post_switchover(
    scope: &str,
    store: &mut dyn DcsStore,
    input: SwitchoverRequestInput,
) -> ApiResult<AcceptedResponse> {
    if input.requested_by.0.trim().is_empty() {
        return Err(ApiError::bad_request("requested_by must be non-empty"));
    }

    let request = SwitchoverRequest {
        requested_by: input.requested_by,
    };
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
    DcsHaWriter::clear_switchover(store, scope)
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
        switchover_requested_by: snapshot
            .value
            .dcs
            .value
            .cache
            .switchover
            .as_ref()
            .map(|switchover| switchover.requested_by.0.clone()),
        member_count: snapshot.value.dcs.value.cache.members.len(),
        dcs_trust: map_dcs_trust(&snapshot.value.dcs.value.trust),
        ha_phase: map_ha_phase(&snapshot.value.ha.value.phase),
        ha_tick: snapshot.value.ha.value.tick,
        ha_decision: map_ha_decision(&snapshot.value.ha.value.decision),
        snapshot_sequence: snapshot.value.sequence,
    }
}

fn map_dcs_trust(value: &DcsTrust) -> DcsTrustResponse {
    match value {
        DcsTrust::FullQuorum => DcsTrustResponse::FullQuorum,
        DcsTrust::FailSafe => DcsTrustResponse::FailSafe,
        DcsTrust::NotTrusted => DcsTrustResponse::NotTrusted,
    }
}

fn map_ha_phase(value: &HaPhase) -> HaPhaseResponse {
    match value {
        HaPhase::Init => HaPhaseResponse::Init,
        HaPhase::WaitingPostgresReachable => HaPhaseResponse::WaitingPostgresReachable,
        HaPhase::WaitingDcsTrusted => HaPhaseResponse::WaitingDcsTrusted,
        HaPhase::WaitingSwitchoverSuccessor => HaPhaseResponse::WaitingSwitchoverSuccessor,
        HaPhase::Replica => HaPhaseResponse::Replica,
        HaPhase::CandidateLeader => HaPhaseResponse::CandidateLeader,
        HaPhase::Primary => HaPhaseResponse::Primary,
        HaPhase::Rewinding => HaPhaseResponse::Rewinding,
        HaPhase::Bootstrapping => HaPhaseResponse::Bootstrapping,
        HaPhase::Fencing => HaPhaseResponse::Fencing,
        HaPhase::FailSafe => HaPhaseResponse::FailSafe,
    }
}

fn map_ha_decision(value: &HaDecision) -> HaDecisionResponse {
    match value {
        HaDecision::NoChange => HaDecisionResponse::NoChange,
        HaDecision::WaitForPostgres {
            start_requested,
            leader_member_id,
        } => HaDecisionResponse::WaitForPostgres {
            start_requested: *start_requested,
            leader_member_id: leader_member_id.as_ref().map(|leader| leader.0.clone()),
        },
        HaDecision::WaitForDcsTrust => HaDecisionResponse::WaitForDcsTrust,
        HaDecision::AttemptLeadership => HaDecisionResponse::AttemptLeadership,
        HaDecision::FollowLeader { leader_member_id } => HaDecisionResponse::FollowLeader {
            leader_member_id: leader_member_id.0.clone(),
        },
        HaDecision::BecomePrimary { promote } => {
            HaDecisionResponse::BecomePrimary { promote: *promote }
        }
        HaDecision::StepDown(plan) => map_step_down_plan(plan),
        HaDecision::RecoverReplica { strategy } => HaDecisionResponse::RecoverReplica {
            strategy: map_recovery_strategy(strategy),
        },
        HaDecision::FenceNode => HaDecisionResponse::FenceNode,
        HaDecision::ReleaseLeaderLease { reason } => HaDecisionResponse::ReleaseLeaderLease {
            reason: map_lease_release_reason(reason),
        },
        HaDecision::EnterFailSafe {
            release_leader_lease,
        } => HaDecisionResponse::EnterFailSafe {
            release_leader_lease: *release_leader_lease,
        },
    }
}

fn map_step_down_plan(value: &StepDownPlan) -> HaDecisionResponse {
    HaDecisionResponse::StepDown {
        reason: map_step_down_reason(&value.reason),
        release_leader_lease: value.release_leader_lease,
        clear_switchover: value.clear_switchover,
        fence: value.fence,
    }
}

fn map_step_down_reason(value: &StepDownReason) -> StepDownReasonResponse {
    match value {
        StepDownReason::Switchover => StepDownReasonResponse::Switchover,
        StepDownReason::ForeignLeaderDetected { leader_member_id } => {
            StepDownReasonResponse::ForeignLeaderDetected {
                leader_member_id: leader_member_id.0.clone(),
            }
        }
    }
}

fn map_recovery_strategy(value: &RecoveryStrategy) -> RecoveryStrategyResponse {
    match value {
        RecoveryStrategy::Rewind { leader_member_id } => RecoveryStrategyResponse::Rewind {
            leader_member_id: leader_member_id.0.clone(),
        },
        RecoveryStrategy::BaseBackup { leader_member_id } => RecoveryStrategyResponse::BaseBackup {
            leader_member_id: leader_member_id.0.clone(),
        },
        RecoveryStrategy::Bootstrap => RecoveryStrategyResponse::Bootstrap,
    }
}

fn map_lease_release_reason(value: &LeaseReleaseReason) -> LeaseReleaseReasonResponse {
    match value {
        LeaseReleaseReason::FencingComplete => LeaseReleaseReasonResponse::FencingComplete,
        LeaseReleaseReason::PostgresUnreachable => LeaseReleaseReasonResponse::PostgresUnreachable,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use crate::{
        api::controller::{delete_switchover, post_switchover, SwitchoverRequestInput},
        dcs::{
            state::SwitchoverRequest,
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        state::MemberId,
    };

    #[derive(Default)]
    struct RecordingStore {
        writes: VecDeque<(String, String)>,
        deletes: VecDeque<String>,
    }

    impl RecordingStore {
        fn pop_write(&mut self) -> Option<(String, String)> {
            self.writes.pop_front()
        }

        fn pop_delete(&mut self) -> Option<String> {
            self.deletes.pop_front()
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            self.writes.push_back((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            self.deletes.push_back(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn switchover_input_denies_unknown_fields() {
        let raw = r#"{"requested_by":"node-a","extra":1}"#;
        let parsed = serde_json::from_str::<SwitchoverRequestInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn post_switchover_writes_typed_record_to_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = post_switchover(
            "scope-a",
            &mut store,
            SwitchoverRequestInput {
                requested_by: MemberId("node-a".to_string()),
            },
        )?;
        assert!(response.accepted);

        let (path, raw) = store
            .pop_write()
            .ok_or_else(|| crate::api::ApiError::internal("expected one DCS write".to_string()))?;
        assert_eq!(path, "/scope-a/switchover");
        let decoded = serde_json::from_str::<SwitchoverRequest>(&raw)
            .map_err(|err| crate::api::ApiError::internal(format!("decode failed: {err}")))?;
        assert_eq!(decoded.requested_by, MemberId("node-a".to_string()));
        Ok(())
    }

    #[test]
    fn post_switchover_rejects_empty_requested_by() {
        let mut store = RecordingStore::default();
        let result = post_switchover(
            "scope-a",
            &mut store,
            SwitchoverRequestInput {
                requested_by: MemberId("".to_string()),
            },
        );
        assert!(matches!(result, Err(crate::api::ApiError::BadRequest(_))));
    }

    #[test]
    fn delete_switchover_deletes_expected_key() -> Result<(), crate::api::ApiError> {
        let mut store = RecordingStore::default();
        let response = delete_switchover("scope-a", &mut store)?;
        assert!(response.accepted);
        assert_eq!(store.pop_delete().as_deref(), Some("/scope-a/switchover"));
        Ok(())
    }
}


===== src/ha/decide.rs =====
use crate::{dcs::state::DcsTrust, process::jobs::ActiveJobKind, state::MemberId};

use super::{
    decision::{
        DecisionFacts, HaDecision, LeaseReleaseReason, PhaseOutcome, ProcessActivity,
        RecoveryStrategy, StepDownPlan, StepDownReason,
    },
    state::{DecideInput, DecideOutput, HaPhase, HaState},
};

pub(crate) fn decide(input: DecideInput) -> DecideOutput {
    let facts = DecisionFacts::from_world(&input.world);
    let current = input.current;
    let outcome = decide_phase(&current, &facts);
    let next = HaState {
        worker: current.worker,
        phase: outcome.next_phase.clone(),
        tick: current.tick.saturating_add(1),
        decision: outcome.decision.clone(),
    };

    DecideOutput { next, outcome }
}

pub(crate) fn decide_phase(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if !matches!(facts.trust, DcsTrust::FullQuorum) {
        if facts.postgres_primary {
            return PhaseOutcome::new(
                HaPhase::FailSafe,
                HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            );
        }
        return PhaseOutcome::new(HaPhase::FailSafe, HaDecision::NoChange);
    }

    match current.phase {
        HaPhase::Init => PhaseOutcome::new(
            HaPhase::WaitingPostgresReachable,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            },
        ),
        HaPhase::WaitingPostgresReachable => decide_waiting_postgres_reachable(facts),
        HaPhase::WaitingDcsTrusted => decide_waiting_dcs_trusted(current, facts),
        HaPhase::WaitingSwitchoverSuccessor => decide_waiting_switchover_successor(facts),
        HaPhase::Replica => decide_replica(facts),
        HaPhase::CandidateLeader => decide_candidate_leader(facts),
        HaPhase::Primary => decide_primary(facts),
        HaPhase::Rewinding => decide_rewinding(facts),
        HaPhase::Bootstrapping => decide_bootstrapping(facts),
        HaPhase::Fencing => decide_fencing(facts),
        HaPhase::FailSafe => decide_fail_safe(facts),
    }
}

fn decide_waiting_postgres_reachable(facts: &DecisionFacts) -> PhaseOutcome {
    if facts.postgres_reachable {
        return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
    }

    if completed_start_postgres(facts) {
        return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
    }

    wait_for_postgres(facts)
}

fn decide_waiting_dcs_trusted(current: &HaState, facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        let released_after_fencing = matches!(
            current.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            }
        );
        if released_after_fencing {
            if let Some(leader_member_id) =
                recovery_leader_member_id(facts).or_else(|| other_leader_record(facts))
            {
                return PhaseOutcome::new(
                    HaPhase::Bootstrapping,
                    HaDecision::RecoverReplica {
                        strategy: RecoveryStrategy::BaseBackup { leader_member_id },
                    },
                );
            }

            return PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust);
        }

        return wait_for_postgres(facts);
    }

    if facts.active_leader_member_id.as_ref() == Some(&facts.self_member_id) {
        return PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary { promote: false },
        );
    }

    match follow_target(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        ),
        None if !facts.postgres_primary => {
            PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust)
        }
        None => PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership),
    }
}

fn decide_waiting_switchover_successor(facts: &DecisionFacts) -> PhaseOutcome {
    if facts
        .leader_member_id
        .as_ref()
        .map(|leader_member_id| leader_member_id == &facts.self_member_id)
        .unwrap_or(true)
    {
        return PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::WaitForDcsTrust,
        );
    }

    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    match follow_target(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        ),
        None => PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::WaitForDcsTrust,
        ),
    }
}

fn decide_replica(facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    if facts.switchover_requested_by.is_some()
        && facts.active_leader_member_id.as_ref() == Some(&facts.self_member_id)
    {
        return PhaseOutcome::new(HaPhase::Replica, HaDecision::NoChange);
    }

    match facts.active_leader_member_id.as_ref() {
        Some(leader_member_id) if leader_member_id == &facts.self_member_id => PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary { promote: true },
        ),
        Some(leader_member_id) if facts.rewind_required => PhaseOutcome::new(
            HaPhase::Rewinding,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: leader_member_id.clone(),
                },
            },
        ),
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader {
                leader_member_id: leader_member_id.clone(),
            },
        ),
        None => PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership),
    }
}

fn decide_candidate_leader(facts: &DecisionFacts) -> PhaseOutcome {
    if !facts.postgres_reachable {
        return wait_for_postgres(facts);
    }

    if facts.i_am_leader {
        return PhaseOutcome::new(
            HaPhase::Primary,
            HaDecision::BecomePrimary {
                promote: !facts.postgres_primary,
            },
        );
    }

    if let Some(leader_member_id) = follow_target(facts) {
        return PhaseOutcome::new(
            HaPhase::Replica,
            HaDecision::FollowLeader { leader_member_id },
        );
    }

    PhaseOutcome::new(HaPhase::CandidateLeader, HaDecision::AttemptLeadership)
}

fn decide_primary(facts: &DecisionFacts) -> PhaseOutcome {
    if facts.switchover_requested_by.is_some() && facts.i_am_leader {
        return PhaseOutcome::new(
            HaPhase::WaitingSwitchoverSuccessor,
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::Switchover,
                release_leader_lease: true,
                clear_switchover: true,
                fence: false,
            }),
        );
    }

    if !facts.postgres_reachable {
        if facts.i_am_leader {
            return PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::PostgresUnreachable,
                },
            );
        }
        return match recovery_leader_member_id(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind { leader_member_id },
                },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        };
    }

    match other_active_leader(facts) {
        Some(leader_member_id) => PhaseOutcome::new(
            HaPhase::Fencing,
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::ForeignLeaderDetected { leader_member_id },
                release_leader_lease: true,
                clear_switchover: false,
                fence: true,
            }),
        ),
        None => {
            if facts.i_am_leader {
                PhaseOutcome::new(HaPhase::Primary, HaDecision::NoChange)
            } else {
                PhaseOutcome::new(HaPhase::Primary, HaDecision::AttemptLeadership)
            }
        }
    }
}

fn decide_rewinding(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.rewind_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => match follow_target(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Replica,
                HaDecision::FollowLeader { leader_member_id },
            ),
            None => PhaseOutcome::new(HaPhase::Replica, HaDecision::NoChange),
        },
        ProcessActivity::IdleFailure => match recovery_after_rewind_failure(facts) {
            Some(strategy) => PhaseOutcome::new(
                HaPhase::Bootstrapping,
                HaDecision::RecoverReplica { strategy },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        },
        ProcessActivity::IdleNoOutcome => match recovery_leader_member_id(facts) {
            Some(leader_member_id) => PhaseOutcome::new(
                HaPhase::Rewinding,
                HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::Rewind { leader_member_id },
                },
            ),
            None => PhaseOutcome::new(HaPhase::Rewinding, HaDecision::NoChange),
        },
    }
}

fn decide_bootstrapping(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.bootstrap_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Bootstrapping, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => wait_for_postgres(facts),
        ProcessActivity::IdleFailure => PhaseOutcome::new(HaPhase::Fencing, HaDecision::FenceNode),
        ProcessActivity::IdleNoOutcome => match recovery_after_rewind_failure(facts) {
            Some(strategy) => PhaseOutcome::new(
                HaPhase::Bootstrapping,
                HaDecision::RecoverReplica { strategy },
            ),
            None => PhaseOutcome::new(HaPhase::Bootstrapping, HaDecision::NoChange),
        },
    }
}

fn decide_fencing(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.fencing_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::Fencing, HaDecision::NoChange),
        ProcessActivity::IdleSuccess => PhaseOutcome::new(
            HaPhase::WaitingDcsTrusted,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
        ),
        ProcessActivity::IdleFailure => PhaseOutcome::new(
            HaPhase::FailSafe,
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
        ),
        ProcessActivity::IdleNoOutcome => {
            PhaseOutcome::new(HaPhase::Fencing, HaDecision::FenceNode)
        }
    }
}

fn decide_fail_safe(facts: &DecisionFacts) -> PhaseOutcome {
    match facts.fencing_activity() {
        ProcessActivity::Running => PhaseOutcome::new(HaPhase::FailSafe, HaDecision::NoChange),
        _ if facts.postgres_primary => decide_primary(facts),
        _ if facts.i_am_leader => PhaseOutcome::new(
            HaPhase::FailSafe,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
        ),
        _ => PhaseOutcome::new(HaPhase::WaitingDcsTrusted, HaDecision::WaitForDcsTrust),
    }
}

fn wait_for_postgres(facts: &DecisionFacts) -> PhaseOutcome {
    PhaseOutcome::new(
        HaPhase::WaitingPostgresReachable,
        HaDecision::WaitForPostgres {
            start_requested: facts.start_postgres_can_be_requested(),
            leader_member_id: recovery_leader_member_id(facts)
                .or_else(|| other_leader_record(facts)),
        },
    )
}

fn recovery_after_rewind_failure(facts: &DecisionFacts) -> Option<RecoveryStrategy> {
    recovery_leader_member_id(facts)
        .map(|leader_member_id| RecoveryStrategy::BaseBackup { leader_member_id })
}

fn recovery_leader_member_id(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .available_primary_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn follow_target(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .available_primary_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn other_leader_record(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .leader_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn other_active_leader(facts: &DecisionFacts) -> Option<MemberId> {
    facts
        .active_leader_member_id
        .clone()
        .filter(|leader_member_id| leader_member_id != &facts.self_member_id)
}

fn completed_start_postgres(facts: &DecisionFacts) -> bool {
    matches!(
        &facts.process_state,
        crate::process::state::ProcessState::Idle {
            last_outcome: Some(
                crate::process::state::JobOutcome::Success {
                    job_kind: ActiveJobKind::StartPostgres,
                    ..
                } | crate::process::state::JobOutcome::Failure {
                    job_kind: ActiveJobKind::StartPostgres,
                    ..
                } | crate::process::state::JobOutcome::Timeout {
                    job_kind: ActiveJobKind::StartPostgres,
                    ..
                }
            ),
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        dcs::state::{
            DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole, SwitchoverRequest,
        },
        ha::{
            decision::{
                HaDecision, LeaseReleaseReason, RecoveryStrategy, StepDownPlan, StepDownReason,
            },
            lower::{
                lower_decision, HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect,
                SafetyEffect, SwitchoverEffect,
            },
            state::{DecideInput, HaPhase, HaState, WorldSnapshot},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::{ActiveJob, ActiveJobKind},
            state::{JobOutcome, ProcessState},
        },
        state::{JobId, MemberId, UnixMillis, Version, Versioned, WorkerStatus},
    };

    use super::decide;

    #[derive(Clone)]
    struct WorldBuilder {
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<MemberId>,
        process: ProcessState,
        members: BTreeMap<MemberId, MemberRecord>,
        switchover_requested_by: Option<MemberId>,
    }

    impl WorldBuilder {
        fn new() -> Self {
            Self {
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                members: BTreeMap::new(),
                switchover_requested_by: None,
            }
        }

        fn with_trust(self, trust: DcsTrust) -> Self {
            Self { trust, ..self }
        }

        fn with_pg(self, pg: PgInfoState) -> Self {
            Self { pg, ..self }
        }

        fn with_process(self, process: ProcessState) -> Self {
            Self { process, ..self }
        }

        fn with_leader(self, leader_member_id: &str) -> Self {
            Self {
                leader: Some(MemberId(leader_member_id.to_string())),
                ..self
            }
        }

        fn with_switchover_request(self, requested_by: &str) -> Self {
            Self {
                switchover_requested_by: Some(MemberId(requested_by.to_string())),
                ..self
            }
        }

        fn with_member(self, record: MemberRecord) -> Self {
            let members = self
                .members
                .into_iter()
                .chain(std::iter::once((record.member_id.clone(), record)))
                .collect();
            Self { members, ..self }
        }

        fn build(self) -> WorldSnapshot {
            world(
                self.trust,
                self.pg,
                self.leader,
                self.process,
                self.members,
                self.switchover_requested_by,
            )
        }
    }

    struct Case {
        name: &'static str,
        current_phase: HaPhase,
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<&'static str>,
        process: ProcessState,
        expected_phase: HaPhase,
        expected_decision: HaDecision,
    }

    #[test]
    fn transition_matrix_cases() {
        let cases = vec![
            Case {
                name: "init moves to waiting postgres",
                current_phase: HaPhase::Init,
                trust: DcsTrust::FullQuorum,
                pg: pg_unknown(SqlStatus::Unknown),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingPostgresReachable,
                expected_decision: HaDecision::WaitForPostgres {
                    start_requested: false,
                    leader_member_id: None,
                },
            },
            Case {
                name: "waiting postgres emits start when unreachable",
                current_phase: HaPhase::WaitingPostgresReachable,
                trust: DcsTrust::FullQuorum,
                pg: pg_unknown(SqlStatus::Unreachable),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingPostgresReachable,
                expected_decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            Case {
                name: "waiting postgres enters dcs trusted when healthy",
                current_phase: HaPhase::WaitingPostgresReachable,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_decision: HaDecision::WaitForDcsTrust,
            },
            Case {
                name: "waiting dcs to replica with known leader",
                current_phase: HaPhase::WaitingDcsTrusted,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(None),
                expected_phase: HaPhase::Replica,
                expected_decision: HaDecision::FollowLeader {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            Case {
                name: "waiting dcs replica without leader stays waiting",
                current_phase: HaPhase::WaitingDcsTrusted,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_decision: HaDecision::WaitForDcsTrust,
            },
            Case {
                name: "candidate becomes primary when lease self",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-a"),
                process: process_idle(None),
                expected_phase: HaPhase::Primary,
                expected_decision: HaDecision::BecomePrimary { promote: true },
            },
            Case {
                name: "primary split brain fences",
                current_phase: HaPhase::Primary,
                trust: DcsTrust::FullQuorum,
                pg: pg_primary(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(None),
                expected_phase: HaPhase::Fencing,
                expected_decision: HaDecision::StepDown(StepDownPlan {
                    reason: StepDownReason::ForeignLeaderDetected {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                    release_leader_lease: true,
                    clear_switchover: false,
                    fence: true,
                }),
            },
            Case {
                name: "no quorum enters fail safe",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::FailSafe,
                expected_decision: HaDecision::NoChange,
            },
            Case {
                name: "rewinding success re-enters replica",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Success {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::PgRewind,
                    finished_at: UnixMillis(10),
                })),
                expected_phase: HaPhase::Replica,
                expected_decision: HaDecision::FollowLeader {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            Case {
                name: "rewinding failure goes bootstrap",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::PgRewind,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(10),
                })),
                expected_phase: HaPhase::Bootstrapping,
                expected_decision: HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::BaseBackup {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                },
            },
            Case {
                name: "rewinding failure without active leader waits",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::PgRewind,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(10),
                })),
                expected_phase: HaPhase::Rewinding,
                expected_decision: HaDecision::NoChange,
            },
            Case {
                name: "bootstrap failure goes fencing",
                current_phase: HaPhase::Bootstrapping,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Timeout {
                    id: JobId("job-1".to_string()),
                    job_kind: ActiveJobKind::Bootstrap,
                    finished_at: UnixMillis(11),
                })),
                expected_phase: HaPhase::Fencing,
                expected_decision: HaDecision::FenceNode,
            },
            Case {
                name: "bootstrapping without active leader emits nothing",
                current_phase: HaPhase::Bootstrapping,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                expected_phase: HaPhase::Bootstrapping,
                expected_decision: HaDecision::NoChange,
            },
            Case {
                name: "fencing success returns waiting dcs",
                current_phase: HaPhase::Fencing,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Success {
                    id: JobId("job-2".to_string()),
                    job_kind: ActiveJobKind::Fencing,
                    finished_at: UnixMillis(12),
                })),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            Case {
                name: "fencing failure enters fail safe",
                current_phase: HaPhase::Fencing,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-2".to_string()),
                    job_kind: ActiveJobKind::Fencing,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(12),
                })),
                expected_phase: HaPhase::FailSafe,
                expected_decision: HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            },
        ];

        for case in cases {
            let input = DecideInput {
                current: HaState {
                    worker: WorkerStatus::Running,
                    phase: case.current_phase.clone(),
                    tick: 41,
                    decision: HaDecision::NoChange,
                },
                world: WorldBuilder::new()
                    .with_trust(case.trust)
                    .with_pg(case.pg.clone())
                    .with_process(process_clone(&case.process))
                    .build_with_optional_leader(case.leader),
            };

            let output = decide(input);
            assert_eq!(
                output.next.phase, case.expected_phase,
                "case: {}",
                case.name
            );
            assert_eq!(
                output.outcome.decision, case.expected_decision,
                "case: {}",
                case.name
            );
            assert_eq!(
                output.next.decision, case.expected_decision,
                "case: {}",
                case.name
            );
            assert_eq!(output.next.tick, 42, "case: {}", case.name);
        }
    }

    #[test]
    fn actions_are_reissued_while_conditions_persist() {
        let current = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::WaitingDcsTrusted,
            tick: 0,
            decision: HaDecision::NoChange,
        };
        let world = WorldBuilder::new()
            .with_pg(pg_primary(SqlStatus::Healthy))
            .build();

        let first = decide(DecideInput {
            current: current.clone(),
            world: world.clone(),
        });
        assert_eq!(
            lower_decision(&first.outcome.decision),
            HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );

        let second = decide(DecideInput {
            current: first.next,
            world,
        });
        assert_eq!(
            lower_decision(&second.outcome.decision),
            HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn fail_safe_holds_without_quorum_and_exits_when_restored() {
        let start = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::FailSafe,
            tick: 100,
            decision: HaDecision::NoChange,
        };

        let held = decide(DecideInput {
            current: start.clone(),
            world: WorldBuilder::new().with_trust(DcsTrust::NotTrusted).build(),
        });
        assert_eq!(held.next.phase, HaPhase::FailSafe);
        assert_eq!(held.outcome.decision, HaDecision::NoChange);

        let recovered = decide(DecideInput {
            current: start,
            world: WorldBuilder::new().with_trust(DcsTrust::FullQuorum).build(),
        });
        assert_eq!(recovered.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(recovered.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn no_quorum_failsafe_with_stale_self_lease_but_stopped_postgres_stays_quiescent() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::FailSafe,
                tick: 44,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_trust(DcsTrust::NotTrusted)
                .with_leader("node-a")
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::FailSafe);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
    }

    #[test]
    fn fail_safe_with_restored_quorum_and_stale_self_lease_retries_release_without_refencing() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::FailSafe,
                tick: 17,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_trust(DcsTrust::FullQuorum)
                .with_leader("node-a")
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::FailSafe);
        assert_eq!(
            output.outcome.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            }
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::ReleaseLeader
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).safety,
            SafetyEffect::None
        );
    }

    #[test]
    fn primary_with_switchover_demotes_releases_and_clears_request() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_leader("node-a")
                .with_switchover_request("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingSwitchoverSuccessor);
        assert_eq!(
            lower_decision(&output.outcome.decision),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::ClearRequest,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Demote,
                safety: SafetyEffect::None,
            }
        );
    }

    #[test]
    fn waiting_switchover_successor_holds_until_new_leader_exists() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingSwitchoverSuccessor,
                tick: 11,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingSwitchoverSuccessor);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_switchover_successor_does_not_restart_while_demote_runs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingSwitchoverSuccessor,
                tick: 12,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_running(ActiveJobKind::Demote))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingSwitchoverSuccessor);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_switchover_successor_follows_new_leader_once_visible() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingSwitchoverSuccessor,
                tick: 13,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .with_leader("node-b")
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.outcome.decision,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            }
        );
    }

    #[test]
    fn waiting_postgres_reachable_with_active_demote_does_not_request_start() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 21,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_running(ActiveJobKind::Demote))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_after_fencing_with_known_leader_reenters_basebackup() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 34,
                decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Bootstrapping);
        assert_eq!(
            output.outcome.decision,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::BaseBackup {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_with_wait_for_dcs_and_known_leader_retries_postgres() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 35,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: true,
                leader_member_id: Some(MemberId("node-b".to_string())),
            }
        );
    }

    #[test]
    fn bootstrapping_success_waits_for_postgres_before_becoming_replica() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Bootstrapping,
                tick: 35,
                decision: HaDecision::RecoverReplica {
                    strategy: RecoveryStrategy::BaseBackup {
                        leader_member_id: MemberId("node-b".to_string()),
                    },
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .with_process(process_idle(Some(JobOutcome::Success {
                    id: JobId("job-basebackup".to_string()),
                    job_kind: ActiveJobKind::BaseBackup,
                    finished_at: UnixMillis(35),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: true,
                leader_member_id: Some(MemberId("node-b".to_string())),
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_without_leader_follows_healthy_primary_member() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 35,
                decision: HaDecision::WaitForDcsTrust,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.outcome.decision,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            }
        );
    }

    #[test]
    fn waiting_dcs_trusted_after_fencing_without_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 35,
                decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_dcs_trusted_after_fencing_uses_stale_foreign_leader_record_for_basebackup() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingDcsTrusted,
                tick: 36,
                decision: HaDecision::ReleaseLeaderLease {
                    reason: LeaseReleaseReason::FencingComplete,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build_with_optional_leader(Some("node-b")),
        });

        assert_eq!(output.next.phase, HaPhase::Bootstrapping);
        assert_eq!(
            output.outcome.decision,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::BaseBackup {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            }
        );
    }

    #[test]
    fn primary_without_leader_reacquires_lease_without_leaving_primary() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::AcquireLeader
        );
    }

    #[test]
    fn replica_with_self_leader_and_pending_switchover_does_not_repromote() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Replica,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-a")
                .with_switchover_request("node-b")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
        assert_eq!(
            lower_decision(&output.outcome.decision),
            HaEffectPlan::default()
        );
    }

    #[test]
    fn rewinding_while_running_emits_nothing() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 8,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_process(process_running(ActiveJobKind::PgRewind))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(lower_decision(&output.outcome.decision).len(), 0);
    }

    #[test]
    fn primary_ignores_unavailable_foreign_leader_record_and_reacquires_lease() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build_with_optional_leader(Some("node-b")),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
    }

    #[test]
    fn primary_outage_without_foreign_leader_waits_in_rewinding_without_self_target() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 9,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
        assert_eq!(lower_decision(&output.outcome.decision).len(), 0);
    }

    #[test]
    fn primary_outage_with_self_leader_releases_lease_before_rewinding() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Unreachable))
                .with_leader("node-a")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(
            output.outcome.decision,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::PostgresUnreachable,
            }
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::ReleaseLeader
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).replication,
            ReplicationEffect::None
        );
    }

    #[test]
    fn rewinding_without_foreign_leader_and_no_process_outcome_emits_nothing() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 10,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Unreachable))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(output.outcome.decision, HaDecision::NoChange);
        assert_eq!(lower_decision(&output.outcome.decision).len(), 0);
    }

    #[test]
    fn rewinding_ignores_stale_start_postgres_failure_until_rewind_runs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 11,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(15),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert_eq!(
            output.outcome.decision,
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            }
        );
    }

    #[test]
    fn waiting_postgres_does_not_reissue_start_while_start_job_is_running() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_running(ActiveJobKind::StartPostgres))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(
            output.outcome.decision,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            }
        );
    }

    #[test]
    fn waiting_postgres_after_failed_start_with_foreign_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 13,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: Some(MemberId("node-b".to_string())),
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-b")
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_failed_start_without_foreign_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 14,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_failed_start_without_leader_uses_healthy_primary_member_waits_for_dcs(
    ) {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 14,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: Some(MemberId("node-b".to_string())),
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_failed_start_as_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 15,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-a")
                .with_process(process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_successful_start_as_follower_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 16,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_process(process_idle(Some(JobOutcome::Success {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn waiting_postgres_after_successful_start_as_leader_waits_for_dcs() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::WaitingPostgresReachable,
                tick: 17,
                decision: HaDecision::WaitForPostgres {
                    start_requested: true,
                    leader_member_id: None,
                },
            },
            world: WorldBuilder::new()
                .with_pg(pg_unknown(SqlStatus::Unreachable))
                .with_leader("node-a")
                .with_process(process_idle(Some(JobOutcome::Success {
                    id: JobId("job-start".to_string()),
                    job_kind: ActiveJobKind::StartPostgres,
                    finished_at: UnixMillis(16),
                })))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::WaitingDcsTrusted);
        assert_eq!(output.outcome.decision, HaDecision::WaitForDcsTrust);
    }

    #[test]
    fn replica_with_unhealthy_leader_becomes_candidate() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Replica,
                tick: 11,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Unknown,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::CandidateLeader);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
    }

    #[test]
    fn candidate_leader_with_unhealthy_foreign_leader_keeps_attempting() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::CandidateLeader,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_leader("node-b")
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Unknown,
                    sql: SqlStatus::Unreachable,
                    readiness: Readiness::NotReady,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::CandidateLeader);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
    }

    #[test]
    fn candidate_leader_without_leader_follows_healthy_primary_member() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::CandidateLeader,
                tick: 12,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_replica(SqlStatus::Healthy))
                .with_member(MemberRecord {
                    member_id: MemberId("node-b".to_string()),
                    postgres_host: "10.0.0.20".to_string(),
                    postgres_port: 5432,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.outcome.decision,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            }
        );
    }

    #[test]
    fn candidate_leader_with_self_lease_and_primary_postgres_skips_promote() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::CandidateLeader,
                tick: 13,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_leader("node-a")
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(
            output.outcome.decision,
            HaDecision::BecomePrimary { promote: false }
        );
    }

    #[test]
    fn decide_is_deterministic_for_identical_inputs() {
        let input = DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 9,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_pg(pg_primary(SqlStatus::Healthy))
                .with_leader("node-b")
                .build(),
        };

        let first = decide(input.clone());
        let second = decide(input.clone());
        let third = decide(input);

        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn non_quorum_trust_always_routes_to_fail_safe() {
        struct FailSafeCase {
            name: &'static str,
            current_phase: HaPhase,
            trust: DcsTrust,
            pg: PgInfoState,
            expected_decision: HaDecision,
        }

        let cases = [
            FailSafeCase {
                name: "primary loses full quorum and fences without lease release",
                current_phase: HaPhase::Primary,
                trust: DcsTrust::NotTrusted,
                pg: pg_primary(SqlStatus::Healthy),
                expected_decision: HaDecision::EnterFailSafe {
                    release_leader_lease: false,
                },
            },
            FailSafeCase {
                name: "replica enters fail safe without extra actions",
                current_phase: HaPhase::Replica,
                trust: DcsTrust::NotTrusted,
                pg: pg_replica(SqlStatus::Healthy),
                expected_decision: HaDecision::NoChange,
            },
            FailSafeCase {
                name: "candidate leader in failsafe trust stays quiescent",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                expected_decision: HaDecision::NoChange,
            },
            FailSafeCase {
                name: "already failsafe replica stays quiescent",
                current_phase: HaPhase::FailSafe,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                expected_decision: HaDecision::NoChange,
            },
        ];

        for case in cases {
            let output = decide(DecideInput {
                current: HaState {
                    worker: WorkerStatus::Running,
                    phase: case.current_phase.clone(),
                    tick: 3,
                    decision: HaDecision::NoChange,
                },
                world: WorldBuilder::new()
                    .with_trust(case.trust)
                    .with_pg(case.pg)
                    .build(),
            });

            assert_eq!(output.next.phase, HaPhase::FailSafe, "case: {}", case.name);
            assert_eq!(
                output.outcome.decision, case.expected_decision,
                "case: {}",
                case.name
            );
            assert_eq!(
                assert_plan_has_no_contradictions(&lower_decision(&output.outcome.decision)),
                Ok(()),
                "case: {}",
                case.name
            );
        }
    }

    #[test]
    fn failsafe_primary_with_full_quorum_returns_to_primary_decision_path() {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::FailSafe,
                tick: 7,
                decision: HaDecision::NoChange,
            },
            world: WorldBuilder::new()
                .with_trust(DcsTrust::FullQuorum)
                .with_pg(pg_primary(SqlStatus::Healthy))
                .build(),
        });

        assert_eq!(output.next.phase, HaPhase::Primary);
        assert_eq!(output.outcome.decision, HaDecision::AttemptLeadership);
        assert_eq!(
            lower_decision(&output.outcome.decision).lease,
            LeaseEffect::AcquireLeader
        );
        assert_eq!(
            lower_decision(&output.outcome.decision).safety,
            SafetyEffect::None
        );
    }

    #[test]
    fn lowered_ha_plans_never_encode_contradictory_actions() {
        let decisions = [
            HaDecision::NoChange,
            HaDecision::WaitForPostgres {
                start_requested: false,
                leader_member_id: None,
            },
            HaDecision::WaitForPostgres {
                start_requested: true,
                leader_member_id: None,
            },
            HaDecision::WaitForDcsTrust,
            HaDecision::AttemptLeadership,
            HaDecision::FollowLeader {
                leader_member_id: MemberId("node-b".to_string()),
            },
            HaDecision::BecomePrimary { promote: false },
            HaDecision::BecomePrimary { promote: true },
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::Switchover,
                release_leader_lease: true,
                clear_switchover: true,
                fence: false,
            }),
            HaDecision::StepDown(StepDownPlan {
                reason: StepDownReason::ForeignLeaderDetected {
                    leader_member_id: MemberId("node-c".to_string()),
                },
                release_leader_lease: true,
                clear_switchover: false,
                fence: true,
            }),
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Rewind {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::BaseBackup {
                    leader_member_id: MemberId("node-b".to_string()),
                },
            },
            HaDecision::RecoverReplica {
                strategy: RecoveryStrategy::Bootstrap,
            },
            HaDecision::FenceNode,
            HaDecision::ReleaseLeaderLease {
                reason: LeaseReleaseReason::FencingComplete,
            },
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
            HaDecision::EnterFailSafe {
                release_leader_lease: true,
            },
        ];

        for decision in decisions {
            let plan = lower_decision(&decision);
            assert_eq!(
                assert_plan_has_no_contradictions(&plan),
                Ok(()),
                "decision: {}",
                decision.label()
            );
        }
    }

    impl WorldBuilder {
        fn build_with_optional_leader(self, leader: Option<&str>) -> WorldSnapshot {
            match leader {
                Some(leader_member_id) => self.with_leader(leader_member_id).build(),
                None => self.build(),
            }
        }
    }

    fn assert_plan_has_no_contradictions(plan: &HaEffectPlan) -> Result<(), String> {
        if matches!(plan.replication, ReplicationEffect::FollowLeader { .. })
            && matches!(plan.postgres, PostgresEffect::Promote)
        {
            return Err("plan cannot follow a leader and promote locally".to_string());
        }

        if matches!(plan.safety, SafetyEffect::SignalFailSafe)
            && (!matches!(plan.replication, ReplicationEffect::None)
                || !matches!(plan.postgres, PostgresEffect::None)
                || !matches!(plan.switchover, SwitchoverEffect::None))
        {
            return Err(
                "fail-safe plan cannot carry replication, postgres, or switchover side effects"
                    .to_string(),
            );
        }

        if matches!(plan.lease, LeaseEffect::AcquireLeader)
            && matches!(plan.postgres, PostgresEffect::Demote)
        {
            return Err("plan cannot acquire the leader lease while demoting postgres".to_string());
        }

        if matches!(plan.safety, SafetyEffect::FenceNode)
            && matches!(plan.postgres, PostgresEffect::Promote)
        {
            return Err("fence plan cannot promote postgres".to_string());
        }

        Ok(())
    }

    fn process_clone(process: &ProcessState) -> ProcessState {
        match process {
            ProcessState::Running { worker, active } => ProcessState::Running {
                worker: worker.clone(),
                active: active.clone(),
            },
            ProcessState::Idle {
                worker,
                last_outcome,
            } => ProcessState::Idle {
                worker: worker.clone(),
                last_outcome: last_outcome.clone(),
            },
        }
    }

    fn process_idle(last_outcome: Option<JobOutcome>) -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome,
        }
    }

    fn process_running(kind: ActiveJobKind) -> ProcessState {
        ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("active-1".to_string()),
                kind,
                started_at: UnixMillis(1),
                deadline_at: UnixMillis(2),
            },
        }
    }

    fn pg_unknown(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Unknown {
            common: pg_common(sql),
        }
    }

    fn pg_primary(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Primary {
            common: pg_common(sql),
            wal_lsn: crate::state::WalLsn(10),
            slots: vec![],
        }
    }

    fn pg_replica(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Replica {
            common: pg_common(sql),
            replay_lsn: crate::state::WalLsn(10),
            follow_lsn: None,
            upstream: None,
        }
    }

    fn pg_common(sql: SqlStatus) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
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
        }
    }

    fn world(
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<MemberId>,
        process: ProcessState,
        members: BTreeMap<MemberId, MemberRecord>,
        switchover_requested_by: Option<MemberId>,
    ) -> WorldSnapshot {
        let cfg = crate::test_harness::runtime_config::sample_runtime_config();

        let leader_record = leader.map(|member_id| LeaderRecord { member_id });

        WorldSnapshot {
            config: Versioned::new(Version(1), UnixMillis(1), cfg.clone()),
            pg: Versioned::new(Version(1), UnixMillis(1), pg),
            dcs: Versioned::new(
                Version(1),
                UnixMillis(1),
                DcsState {
                    worker: WorkerStatus::Running,
                    trust,
                    cache: DcsCache {
                        members,
                        leader: leader_record,
                        switchover: switchover_requested_by
                            .map(|requested_by| SwitchoverRequest { requested_by }),
                        config: cfg,
                        init_lock: None,
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
            ),
            process: Versioned::new(Version(1), UnixMillis(1), process),
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


===== src/cli/client.rs =====
use std::time::Duration;

use reqwest::{Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) use crate::api::{AcceptedResponse, HaDecisionResponse, HaStateResponse};
use crate::cli::error::CliError;

#[derive(Clone, Debug)]
pub struct CliApiClient {
    base_url: Url,
    http: reqwest::Client,
    read_token: Option<String>,
    admin_token: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthRole {
    Read,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct SwitchoverRequestInput {
    requested_by: String,
}

impl CliApiClient {
    pub fn new(
        base_url: String,
        timeout_ms: u64,
        read_token: Option<String>,
        admin_token: Option<String>,
    ) -> Result<Self, CliError> {
        let base_url = Url::parse(base_url.trim())
            .map_err(|err| CliError::RequestBuild(format!("invalid --base-url value: {err}")))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .pool_max_idle_per_host(0)
            .build()
            .map_err(|err| CliError::RequestBuild(format!("build http client failed: {err}")))?;

        Ok(Self {
            base_url,
            http,
            read_token: normalize_token(read_token),
            admin_token: normalize_token(admin_token),
        })
    }

    pub async fn get_ha_state(&self) -> Result<HaStateResponse, CliError> {
        self.send_json_no_body(Method::GET, "/ha/state", AuthRole::Read, StatusCode::OK)
            .await
    }

    pub async fn delete_switchover(&self) -> Result<AcceptedResponse, CliError> {
        self.send_json_no_body(
            Method::DELETE,
            "/ha/switchover",
            AuthRole::Admin,
            StatusCode::ACCEPTED,
        )
        .await
    }

    pub async fn post_switchover(
        &self,
        requested_by: String,
    ) -> Result<AcceptedResponse, CliError> {
        let body = SwitchoverRequestInput { requested_by };
        self.send_json_with_body(
            Method::POST,
            "/switchover",
            AuthRole::Admin,
            &body,
            StatusCode::ACCEPTED,
        )
        .await
    }

    async fn send_json_no_body<T>(
        &self,
        method: Method,
        path: &str,
        role: AuthRole,
        expected_status: StatusCode,
    ) -> Result<T, CliError>
    where
        T: DeserializeOwned,
    {
        let url = self
            .base_url
            .join(path)
            .map_err(|err| CliError::RequestBuild(format!("compose URL failed: {err}")))?;
        let mut request = self.http.request(method, url);
        if let Some(token) = self.token_for(role) {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| CliError::Transport(err.to_string()))?;

        read_json_response(response, expected_status).await
    }

    async fn send_json_with_body<T, B>(
        &self,
        method: Method,
        path: &str,
        role: AuthRole,
        body: &B,
        expected_status: StatusCode,
    ) -> Result<T, CliError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .base_url
            .join(path)
            .map_err(|err| CliError::RequestBuild(format!("compose URL failed: {err}")))?;
        let mut request = self.http.request(method, url);
        if let Some(token) = self.token_for(role) {
            request = request.bearer_auth(token);
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(|err| CliError::Transport(err.to_string()))?;

        read_json_response(response, expected_status).await
    }

    fn token_for(&self, role: AuthRole) -> Option<&str> {
        match role {
            AuthRole::Read => self.read_token.as_deref().or(self.admin_token.as_deref()),
            AuthRole::Admin => self.admin_token.as_deref(),
        }
    }
}

async fn read_json_response<T>(
    response: reqwest::Response,
    expected_status: StatusCode,
) -> Result<T, CliError>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status != expected_status {
        let body = match response.text().await {
            Ok(value) => value,
            Err(err) => format!("<failed to read response body: {err}>"),
        };
        return Err(CliError::ApiStatus {
            status: status.as_u16(),
            body,
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|err| CliError::Decode(err.to_string()))
}

fn normalize_token(raw: Option<String>) -> Option<String> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use crate::cli::client::{CliApiClient, CliError, HaDecisionResponse};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RecordedRequest {
        method: String,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    }

    #[tokio::test]
    async fn state_request_uses_read_token_when_configured() -> Result<(), CliError> {
        let response_body = r#"{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":null,"switchover_requested_by":null,"member_count":1,"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{"kind":"become_primary","promote":true},"snapshot_sequence":10}"#;
        let (addr, handle) = spawn_server(http_response(200, response_body)).await?;

        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("read-token".to_string()),
            Some("admin-token".to_string()),
        )?;
        let state = client.get_ha_state().await?;
        assert_eq!(state.cluster_name, "cluster-a");
        assert_eq!(
            state.ha_decision,
            HaDecisionResponse::BecomePrimary { promote: true }
        );

        let request = handle_request(handle).await?;
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/ha/state");
        assert_header(&request.headers, "authorization", "Bearer read-token")?;
        Ok(())
    }

    #[tokio::test]
    async fn state_request_falls_back_to_admin_token_when_read_missing() -> Result<(), CliError> {
        let response_body = r#"{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":null,"switchover_requested_by":null,"member_count":1,"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{"kind":"become_primary","promote":true},"snapshot_sequence":10}"#;
        let (addr, handle) = spawn_server(http_response(200, response_body)).await?;

        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            None,
            Some("admin-token".to_string()),
        )?;
        let _ = client.get_ha_state().await?;

        let request = handle_request(handle).await?;
        assert_header(&request.headers, "authorization", "Bearer admin-token")?;
        Ok(())
    }

    #[tokio::test]
    async fn switchover_clear_uses_delete_endpoint() -> Result<(), CliError> {
        let (addr, handle) = spawn_server(http_response(202, r#"{"accepted":true}"#)).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let _ = client.delete_switchover().await?;
        let request = handle_request(handle).await?;
        assert_eq!(request.method, "DELETE");
        assert_eq!(request.path, "/ha/switchover");
        assert_header(&request.headers, "authorization", "Bearer admin")?;
        Ok(())
    }

    #[tokio::test]
    async fn non_2xx_maps_to_api_status_error() -> Result<(), CliError> {
        let (addr, _handle) = spawn_server(http_response(403, "forbidden")).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let result = client.get_ha_state().await;
        match result {
            Err(CliError::ApiStatus { status, body }) => {
                assert_eq!(status, 403);
                assert_eq!(body, "forbidden");
            }
            Err(other) => {
                return Err(CliError::Decode(format!(
                    "expected ApiStatus error, got {other}"
                )));
            }
            Ok(_) => {
                return Err(CliError::Decode(
                    "expected failure for non-2xx response".to_string(),
                ));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn malformed_json_maps_to_decode_error() -> Result<(), CliError> {
        let (addr, _handle) = spawn_server(http_response(200, "{not-json")).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let result = client.get_ha_state().await;
        match result {
            Err(CliError::Decode(_)) => Ok(()),
            Err(other) => Err(CliError::Decode(format!(
                "expected decode error, got {other}"
            ))),
            Ok(_) => Err(CliError::Decode(
                "expected decode failure for malformed json".to_string(),
            )),
        }
    }

    #[tokio::test]
    async fn connection_refused_maps_to_transport_error() -> Result<(), CliError> {
        let addr = reserve_unused_addr().await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            200,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let result = client.get_ha_state().await;
        match result {
            Err(CliError::Transport(_)) => Ok(()),
            Err(other) => Err(CliError::Decode(format!(
                "expected transport error, got {other}"
            ))),
            Ok(_) => Err(CliError::Decode(
                "expected transport failure on unused port".to_string(),
            )),
        }
    }

    async fn reserve_unused_addr() -> Result<SocketAddr, CliError> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| CliError::Transport(format!("bind failed: {err}")))?;
        listener
            .local_addr()
            .map_err(|err| CliError::Transport(format!("local_addr failed: {err}")))
    }

    async fn spawn_server(
        response: String,
    ) -> Result<
        (
            SocketAddr,
            tokio::task::JoinHandle<Result<RecordedRequest, CliError>>,
        ),
        CliError,
    > {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| CliError::Transport(format!("bind failed: {err}")))?;
        let addr = listener
            .local_addr()
            .map_err(|err| CliError::Transport(format!("local_addr failed: {err}")))?;

        let handle = tokio::spawn(async move {
            let (mut stream, _peer) = listener
                .accept()
                .await
                .map_err(|err| CliError::Transport(format!("accept failed: {err}")))?;
            let request = read_http_request(&mut stream).await?;
            stream
                .write_all(response.as_bytes())
                .await
                .map_err(|err| CliError::Transport(format!("response write failed: {err}")))?;
            stream
                .shutdown()
                .await
                .map_err(|err| CliError::Transport(format!("shutdown failed: {err}")))?;
            Ok(request)
        });

        Ok((addr, handle))
    }

    async fn handle_request(
        handle: tokio::task::JoinHandle<Result<RecordedRequest, CliError>>,
    ) -> Result<RecordedRequest, CliError> {
        match handle.await {
            Ok(result) => result,
            Err(err) => Err(CliError::Transport(format!("server task failed: {err}"))),
        }
    }

    async fn read_http_request(
        stream: &mut tokio::net::TcpStream,
    ) -> Result<RecordedRequest, CliError> {
        let mut buffer = Vec::new();
        let mut temp = [0u8; 1024];

        loop {
            let read = stream
                .read(&mut temp)
                .await
                .map_err(|err| CliError::Transport(format!("request read failed: {err}")))?;
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);

            if let Some(header_end) = find_header_end(&buffer) {
                let content_length = parse_content_length(&buffer[..header_end])?;
                if buffer.len() >= header_end + content_length {
                    break;
                }
            }
        }

        parse_http_request(&buffer)
    }

    fn parse_http_request(buffer: &[u8]) -> Result<RecordedRequest, CliError> {
        let header_end = find_header_end(buffer).ok_or_else(|| {
            CliError::Decode("request parse failed: missing header terminator".to_string())
        })?;

        let header_text = std::str::from_utf8(&buffer[..header_end]).map_err(|err| {
            CliError::Decode(format!("request parse failed: invalid utf8 headers: {err}"))
        })?;
        let mut lines = header_text.split("\r\n");
        let request_line = lines.next().ok_or_else(|| {
            CliError::Decode("request parse failed: missing request line".to_string())
        })?;

        let mut request_parts = request_line.split_whitespace();
        let method = request_parts
            .next()
            .ok_or_else(|| CliError::Decode("missing request method".to_string()))?
            .to_string();
        let path = request_parts
            .next()
            .ok_or_else(|| CliError::Decode("missing request path".to_string()))?
            .to_string();

        let mut headers = Vec::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.push((name.trim().to_string(), value.trim().to_string()));
            }
        }

        let content_length = parse_content_length(&buffer[..header_end])?;
        let body_end = header_end
            .checked_add(content_length)
            .ok_or_else(|| CliError::Decode("request body length overflow".to_string()))?;
        if body_end > buffer.len() {
            return Err(CliError::Decode(
                "request parse failed: body shorter than content-length".to_string(),
            ));
        }

        Ok(RecordedRequest {
            method,
            path,
            headers,
            body: buffer[header_end..body_end].to_vec(),
        })
    }

    fn parse_content_length(headers: &[u8]) -> Result<usize, CliError> {
        let text = std::str::from_utf8(headers)
            .map_err(|err| CliError::Decode(format!("header utf8 decode failed: {err}")))?;
        for line in text.split("\r\n") {
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") {
                    let parsed = value.trim().parse::<usize>().map_err(|err| {
                        CliError::Decode(format!("content-length parse failed: {err}"))
                    })?;
                    return Ok(parsed);
                }
            }
        }
        Ok(0)
    }

    fn find_header_end(buffer: &[u8]) -> Option<usize> {
        buffer
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|value| value + 4)
    }

    fn http_response(status_code: u16, body: &str) -> String {
        let reason = match status_code {
            200 => "OK",
            202 => "Accepted",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Status",
        };
        format!(
            "HTTP/1.1 {status_code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn assert_header(
        headers: &[(String, String)],
        expected_name: &str,
        expected_value: &str,
    ) -> Result<(), CliError> {
        let found = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(expected_name))
            .map(|(_, value)| value.as_str());
        match found {
            Some(value) if value == expected_value => Ok(()),
            Some(value) => Err(CliError::Decode(format!(
                "header mismatch for {expected_name}: expected {expected_value}, got {value}"
            ))),
            None => Err(CliError::Decode(format!(
                "missing required header {expected_name}"
            ))),
        }
    }
}


===== src/cli/args.rs =====
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "pgtuskmasterctl")]
#[command(about = "HA admin CLI for PGTuskMaster API")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    #[arg(long, env = "PGTUSKMASTER_READ_TOKEN")]
    pub read_token: Option<String>,
    #[arg(long, env = "PGTUSKMASTER_ADMIN_TOKEN")]
    pub admin_token: Option<String>,
    #[arg(long, default_value_t = 5_000)]
    pub timeout_ms: u64,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub output: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Ha(HaArgs),
}

#[derive(Clone, Debug, Args)]
pub struct HaArgs {
    #[command(subcommand)]
    pub command: HaCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum HaCommand {
    State,
    Switchover(SwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct SwitchoverArgs {
    #[command(subcommand)]
    pub command: SwitchoverCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SwitchoverCommand {
    Clear,
    Request(RequestSwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct RequestSwitchoverArgs {
    #[arg(long)]
    pub requested_by: String,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::args::{Cli, Command, HaCommand, OutputFormat, SwitchoverCommand};

    #[test]
    fn parse_ha_state_with_defaults() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "http://127.0.0.1:8080");
        assert_eq!(cli.timeout_ms, 5_000);
        assert_eq!(cli.output, OutputFormat::Json);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::State => Ok(()),
                _ => Err("expected ha state command".to_string()),
            },
        }
    }

    #[test]
    fn parse_requires_requested_by_for_switchover_request() {
        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "switchover", "request"]);
        assert!(parsed.is_err(), "requested-by is required");
    }

    #[test]
    fn parse_full_switchover_write_command() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "--base-url",
            "https://cluster.example",
            "--timeout-ms",
            "1234",
            "--output",
            "text",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-a",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "https://cluster.example");
        assert_eq!(cli.timeout_ms, 1234);
        assert_eq!(cli.output, OutputFormat::Text);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-a");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_switchover_request() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-b",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-b");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_env_token_fallbacks() -> Result<(), String> {
        let read_var = "PGTUSKMASTER_READ_TOKEN";
        let admin_var = "PGTUSKMASTER_ADMIN_TOKEN";

        std::env::set_var(read_var, "reader");
        std::env::set_var(admin_var, "admin");

        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"));

        std::env::remove_var(read_var);
        std::env::remove_var(admin_var);

        let cli = parsed?;
        assert_eq!(cli.read_token.as_deref(), Some("reader"));
        assert_eq!(cli.admin_token.as_deref(), Some("admin"));
        Ok(())
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


===== tests/ha/support/observer.rs =====
use pgtuskmaster_rust::{api::HaPhaseResponse, api::HaStateResponse, state::WorkerError};

#[derive(Clone, Default, serde::Serialize)]
pub struct HaObservationStats {
    pub sample_count: u64,
    pub api_error_count: u64,
    pub max_concurrent_primaries: usize,
    pub leader_change_count: u64,
    pub failsafe_sample_count: u64,
    pub recent_samples: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct HaObserverConfig {
    pub min_successful_samples: u64,
    pub ring_capacity: usize,
}

pub struct HaInvariantObserver {
    config: HaObserverConfig,
    stats: HaObservationStats,
    poll_attempts: u64,
    poll_errors: u64,
    last_poll_error: Option<String>,
    last_leader_signature: Option<String>,
}

impl HaInvariantObserver {
    pub fn new(config: HaObserverConfig) -> Self {
        Self {
            config,
            stats: HaObservationStats::default(),
            poll_attempts: 0,
            poll_errors: 0,
            last_poll_error: None,
            last_leader_signature: None,
        }
    }

    pub fn record_poll_attempt(&mut self) {
        self.poll_attempts = self.poll_attempts.saturating_add(1);
    }

    pub fn record_api_states(
        &mut self,
        states: &[HaStateResponse],
        errors: &[String],
    ) -> Result<(), WorkerError> {
        self.stats.api_error_count = self
            .stats
            .api_error_count
            .saturating_add(len_to_u64(errors.len()));

        if states.is_empty() {
            if !errors.is_empty() {
                self.push_recent(format!("api_blind_spot: {}", errors.join(" | ")));
            }
            return Ok(());
        }

        self.stats.sample_count = self.stats.sample_count.saturating_add(1);
        let primary_count = states
            .iter()
            .filter(|state| state.ha_phase == HaPhaseResponse::Primary)
            .count();
        self.stats.max_concurrent_primaries =
            self.stats.max_concurrent_primaries.max(primary_count);

        let mut leaders = states
            .iter()
            .filter_map(|state| state.leader.clone())
            .collect::<Vec<_>>();
        leaders.sort();
        leaders.dedup();
        let leader_signature = leaders.join("|");
        if self
            .last_leader_signature
            .as_deref()
            .map(|prior| prior != leader_signature.as_str())
            .unwrap_or(false)
        {
            self.stats.leader_change_count = self.stats.leader_change_count.saturating_add(1);
        }
        self.last_leader_signature = Some(leader_signature);

        if states
            .iter()
            .all(|state| state.ha_phase == HaPhaseResponse::FailSafe)
        {
            self.stats.failsafe_sample_count = self.stats.failsafe_sample_count.saturating_add(1);
        }

        let mut fragments = states
            .iter()
            .map(|state| {
                let leader = state.leader.as_deref().unwrap_or("none");
                format!(
                    "{}:{}:leader={leader}",
                    state.self_member_id, state.ha_phase
                )
            })
            .collect::<Vec<_>>();
        fragments.extend(errors.iter().map(|error| format!("api_error={error}")));
        self.push_recent(fragments.join(", "));

        if primary_count > 1 {
            return Err(WorkerError::Message(format!(
                "split-brain detected: more than one primary; observations={} errors={}",
                states
                    .iter()
                    .map(|state| format!("{}:{}", state.self_member_id, state.ha_phase))
                    .collect::<Vec<_>>()
                    .join(","),
                summarize_errors(errors)
            )));
        }

        Ok(())
    }

    pub fn record_sql_roles(
        &mut self,
        roles: &[(String, String)],
        errors: &[String],
    ) -> Result<(), WorkerError> {
        if roles.is_empty() {
            if !errors.is_empty() {
                self.push_recent(format!("sql_blind_spot: {}", errors.join(" | ")));
            }
            return Ok(());
        }

        self.stats.sample_count = self.stats.sample_count.saturating_add(1);
        let primary_count = roles.iter().filter(|(_, role)| role == "primary").count();
        self.stats.max_concurrent_primaries =
            self.stats.max_concurrent_primaries.max(primary_count);

        let mut fragments = roles
            .iter()
            .map(|(node_id, role)| format!("{node_id}:{role}"))
            .collect::<Vec<_>>();
        fragments.extend(errors.iter().map(|error| format!("sql_error={error}")));
        self.push_recent(format!("sql_roles=[{}]", fragments.join(", ")));

        if primary_count > 1 {
            return Err(WorkerError::Message(format!(
                "split-brain detected via SQL roles: observations={} errors={}",
                roles
                    .iter()
                    .map(|(node_id, role)| format!("{node_id}:{role}"))
                    .collect::<Vec<_>>()
                    .join(","),
                summarize_errors(errors)
            )));
        }

        Ok(())
    }

    pub fn record_observation_gap(&mut self, api_errors: &[String], sql_errors: &[String]) {
        self.poll_errors = self.poll_errors.saturating_add(1);
        let message = format!(
            "api_errors={}; sql_errors={}",
            summarize_errors(api_errors),
            summarize_errors(sql_errors)
        );
        self.last_poll_error = Some(message.clone());
        self.push_recent(format!("observation_gap:{message}"));
    }

    pub fn record_transport_error(&mut self, error: impl Into<String>) {
        self.poll_errors = self.poll_errors.saturating_add(1);
        let message = error.into();
        self.last_poll_error = Some(message.clone());
        self.push_recent(format!("transport_error:{message}"));
    }

    pub fn stats(&self) -> &HaObservationStats {
        &self.stats
    }

    pub fn into_stats(self) -> HaObservationStats {
        self.stats
    }

    pub fn finalize_no_dual_primary_window(&self) -> Result<(), WorkerError> {
        if self.stats.sample_count < self.config.min_successful_samples {
            let detail = self.last_poll_error.as_deref().unwrap_or("none");
            return Err(WorkerError::Message(format!(
                "insufficient evidence for split-brain window assertion: successful_samples={} min_successful_samples={} poll_attempts={} poll_errors={} last_poll_error={} recent_samples={}",
                self.stats.sample_count,
                self.config.min_successful_samples,
                self.poll_attempts,
                self.poll_errors,
                detail,
                summarize_recent_samples(&self.stats.recent_samples),
            )));
        }

        assert_no_dual_primary_in_samples(self.stats(), self.config.min_successful_samples)
    }

    fn push_recent(&mut self, sample: String) {
        if self.config.ring_capacity == 0 {
            return;
        }
        if self.stats.recent_samples.len() >= self.config.ring_capacity {
            let _ = self.stats.recent_samples.remove(0);
        }
        self.stats.recent_samples.push(sample);
    }
}

pub fn assert_no_dual_primary_in_samples(
    stats: &HaObservationStats,
    min_successful_samples: u64,
) -> Result<(), WorkerError> {
    if stats.sample_count < min_successful_samples {
        return Err(WorkerError::Message(format!(
            "insufficient HA sample evidence: sample_count={} min_successful_samples={} api_error_count={} recent_samples={}",
            stats.sample_count,
            min_successful_samples,
            stats.api_error_count,
            summarize_recent_samples(&stats.recent_samples),
        )));
    }
    if stats.max_concurrent_primaries > 1 {
        return Err(WorkerError::Message(format!(
            "dual primary observed during sampled window; max_concurrent_primaries={} recent_samples={}",
            stats.max_concurrent_primaries,
            summarize_recent_samples(&stats.recent_samples),
        )));
    }
    Ok(())
}

fn len_to_u64(value: usize) -> u64 {
    u64::try_from(value)
        .ok()
        .map_or(u64::MAX, core::convert::identity)
}

fn summarize_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        "none".to_string()
    } else {
        errors.join(" | ")
    }
}

fn summarize_recent_samples(samples: &[String]) -> String {
    if samples.is_empty() {
        "none".to_string()
    } else {
        samples.join(" || ")
    }
}

#[cfg(test)]
mod unit_tests {
    use super::{
        assert_no_dual_primary_in_samples, HaInvariantObserver, HaObservationStats,
        HaObserverConfig,
    };
    use pgtuskmaster_rust::api::{
        DcsTrustResponse, HaDecisionResponse, HaPhaseResponse, HaStateResponse,
    };

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn ha_state(member_id: &str, phase: HaPhaseResponse, leader: Option<&str>) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-e2e".to_string(),
            scope: "scope-ha-e2e".to_string(),
            self_member_id: member_id.to_string(),
            leader: leader.map(ToString::to_string),
            switchover_requested_by: None,
            member_count: 3,
            dcs_trust: DcsTrustResponse::FullQuorum,
            ha_phase: phase,
            ha_tick: 1,
            ha_decision: HaDecisionResponse::NoChange,
            snapshot_sequence: 1,
        }
    }

    #[test]
    fn zero_sample_finalization_fails_closed() -> TestResult {
        let observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        let result = observer.finalize_no_dual_primary_window();
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected finalization to fail with zero samples",
            )));
        }
        Ok(())
    }

    #[test]
    fn insufficient_sample_threshold_fails() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 2,
            ring_capacity: 4,
        });
        observer.record_poll_attempt();
        observer.record_api_states(
            &[ha_state("node-1", HaPhaseResponse::Primary, Some("node-1"))],
            &[],
        )?;
        let result = observer.finalize_no_dual_primary_window();
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected finalization to fail when the minimum sample threshold is not met",
            )));
        }
        Ok(())
    }

    #[test]
    fn successful_finalization_with_enough_samples() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 2,
            ring_capacity: 4,
        });
        for _ in 0..2 {
            observer.record_poll_attempt();
            observer.record_api_states(
                &[ha_state("node-1", HaPhaseResponse::Primary, Some("node-1"))],
                &[],
            )?;
        }
        observer.finalize_no_dual_primary_window()?;
        Ok(())
    }

    #[test]
    fn dual_primary_sample_is_rejected() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        observer.record_poll_attempt();
        let result = observer.record_api_states(
            &[
                ha_state("node-1", HaPhaseResponse::Primary, Some("node-1")),
                ha_state("node-2", HaPhaseResponse::Primary, Some("node-2")),
            ],
            &[],
        );
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected dual-primary sample to fail",
            )));
        }
        Ok(())
    }

    #[test]
    fn standalone_sample_assertion_rejects_dual_primary_stats() -> TestResult {
        let stats = HaObservationStats {
            sample_count: 3,
            api_error_count: 1,
            max_concurrent_primaries: 2,
            leader_change_count: 0,
            failsafe_sample_count: 0,
            recent_samples: vec!["node-1:Primary,node-2:Primary".to_string()],
        };
        let result = assert_no_dual_primary_in_samples(&stats, 1);
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected dual-primary stats assertion to fail",
            )));
        }
        Ok(())
    }

    #[test]
    fn observer_transport_and_stats_helpers_are_reachable() {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        observer.record_transport_error("synthetic");
        let stats = observer.into_stats();
        assert_eq!(stats.recent_samples.len(), 1);
    }
}


===== tests/ha/support/multi_node.rs =====
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use clap::Parser;
use tokio::task::JoinHandle;

use super::observer::{
    assert_no_dual_primary_in_samples, HaInvariantObserver, HaObservationStats, HaObserverConfig,
};

use pgtuskmaster_rust::{
    api::{AcceptedResponse as CliAcceptedResponse, HaStateResponse},
    cli::{self, args::Cli, client::CliApiClient, error::CliError},
    state::WorkerError,
    test_harness::ha_e2e,
};

use pgtuskmaster_rust::test_harness::ha_e2e::handle::TestClusterHandle;

struct ClusterFixture {
    _guard: pgtuskmaster_rust::test_harness::namespace::NamespaceGuard,
    pg_ctl_bin: PathBuf,
    psql_bin: PathBuf,
    superuser_username: String,
    superuser_dbname: String,
    etcd: Option<pgtuskmaster_rust::test_harness::etcd3::EtcdClusterHandle>,
    nodes: Vec<ha_e2e::NodeHandle>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    timeline: Vec<String>,
}

const E2E_COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_SQL_WORKLOAD_COMMAND_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT: Duration = Duration::from_secs(1);
const E2E_PG_STOP_TIMEOUT: Duration = Duration::from_secs(10);
const E2E_HTTP_STEP_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_BOOTSTRAP_PRIMARY_TIMEOUT: Duration = Duration::from_secs(45);
const E2E_SCENARIO_TIMEOUT: Duration = Duration::from_secs(300);
const E2E_API_READINESS_TIMEOUT: Duration = Duration::from_secs(120);
const E2E_STABLE_PRIMARY_API_POLL_INTERVAL: Duration = Duration::from_millis(100);
const E2E_STABLE_PRIMARY_SQL_POLL_INTERVAL: Duration = Duration::from_millis(200);
const E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL: Duration = Duration::from_millis(75);
const E2E_NO_QUORUM_OBSERVATION_TIMEOUT: Duration = Duration::from_secs(3);
const E2E_NO_QUORUM_LOG_INTERVAL: Duration = Duration::from_secs(5);
const E2E_NO_QUORUM_RETRY_INTERVAL: Duration = Duration::from_millis(100);
const E2E_SQL_RETRY_INTERVAL: Duration = Duration::from_millis(200);
const E2E_STABLE_PRIMARY_STRICT_TIMEOUT_CAP: Duration = Duration::from_secs(45);
const E2E_STABLE_PRIMARY_API_FALLBACK_TIMEOUT_CAP: Duration = Duration::from_secs(45);
const E2E_STABLE_PRIMARY_SQL_FALLBACK_TIMEOUT_CAP: Duration = Duration::from_secs(90);
const E2E_STABLE_PRIMARY_STRICT_CONSECUTIVE_CAP: usize = 3;
const E2E_STABLE_PRIMARY_RELAXED_CONSECUTIVE_CAP: usize = 2;
const E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS: u64 = 250;
const E2E_STRESS_SAMPLE_INTERVAL: Duration = Duration::from_millis(150);
const E2E_STRESS_WORKLOAD_STOP_TIMEOUT: Duration = Duration::from_secs(2);
const E2E_NO_QUORUM_WORKLOAD_STOP_TIMEOUT: Duration = Duration::from_millis(200);
const E2E_SWITCHOVER_RETRY_BACKOFF: Duration = Duration::from_millis(500);
const E2E_PRIMARY_CONVERGENCE_TIMEOUT: Duration = Duration::from_secs(60);
const E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT: Duration = Duration::from_secs(90);
const E2E_SQL_REPLICATION_ASSERT_TIMEOUT: Duration = Duration::from_secs(20);
const E2E_SHORT_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(3);
const E2E_LONG_NO_DUAL_PRIMARY_WINDOW: Duration = Duration::from_secs(10);
const E2E_STRESS_WORKLOAD_SETTLE_WAIT: Duration = Duration::from_secs(3);
const E2E_STRESS_SHORT_OBSERVATION_WINDOW: Duration = Duration::from_secs(8);
const E2E_STRESS_LONG_OBSERVATION_WINDOW: Duration = Duration::from_secs(10);
const E2E_POST_TRANSITION_SQL_TIMEOUT: Duration = Duration::from_secs(30);
const E2E_TABLE_INTEGRITY_TIMEOUT: Duration = Duration::from_secs(90);
const E2E_LOADED_FAILOVER_TIMEOUT: Duration = Duration::from_secs(180);
const STRESS_ARTIFACT_DIR: &str = ".ralph/evidence/27-e2e-ha-stress";
const STRESS_SUMMARY_SCHEMA_VERSION: u32 = 1;

static E2E_UNIQUE_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy)]
struct StablePrimaryWaitPlan<'a> {
    context: &'a str,
    timeout: Duration,
    excluded_primary: Option<&'a str>,
    required_consecutive: usize,
    fallback_timeout: Duration,
    fallback_required_consecutive: usize,
    min_observed_nodes: usize,
}

fn unique_e2e_token() -> Result<String, WorkerError> {
    let now = ha_e2e::util::unix_now()?.0;
    let seq = E2E_UNIQUE_SEQ.fetch_add(1, Ordering::Relaxed);
    Ok(format!("{now}-{seq}"))
}

fn e2e_http_timeout_ms() -> Result<u64, WorkerError> {
    u64::try_from(E2E_HTTP_STEP_TIMEOUT.as_millis())
        .map_err(|_| WorkerError::Message("e2e HTTP timeout does not fit into u64".to_string()))
}

#[derive(Clone)]
struct SqlWorkloadSpec {
    scenario_name: String,
    table_name: String,
    worker_count: usize,
    run_interval_ms: u64,
}

impl SqlWorkloadSpec {
    fn interval(&self) -> Duration {
        Duration::from_millis(self.run_interval_ms.max(1))
    }
}

#[derive(Clone)]
struct SqlWorkloadTarget {
    node_id: String,
    port: u16,
}

#[derive(Clone)]
struct SqlWorkloadCtx {
    psql_bin: PathBuf,
    superuser_username: String,
    superuser_dbname: String,
    scenario_name: String,
    table_name: String,
    interval: Duration,
    targets: Vec<SqlWorkloadTarget>,
}

struct SqlWorkloadHandle {
    spec: SqlWorkloadSpec,
    started_at_unix_ms: u64,
    stop_tx: tokio::sync::watch::Sender<bool>,
    joins: Vec<JoinHandle<Result<SqlWorkloadWorkerStats, WorkerError>>>,
}

#[derive(Default, serde::Serialize)]
struct SqlWorkloadWorkerStats {
    worker_id: usize,
    attempted_writes: u64,
    committed_writes: u64,
    attempted_reads: u64,
    read_successes: u64,
    transient_failures: u64,
    fencing_failures: u64,
    hard_failures: u64,
    commit_timestamp_capture_failures: u64,
    write_latency_total_ms: u64,
    write_latency_max_ms: u64,
    committed_keys: Vec<String>,
    committed_at_unix_ms: Vec<u64>,
    last_error: Option<String>,
}

#[derive(Default, serde::Serialize)]
struct SqlWorkloadStats {
    scenario_name: String,
    table_name: String,
    worker_count: usize,
    started_at_unix_ms: u64,
    finished_at_unix_ms: u64,
    duration_ms: u64,
    attempted_writes: u64,
    committed_writes: u64,
    attempted_reads: u64,
    read_successes: u64,
    transient_failures: u64,
    fencing_failures: u64,
    hard_failures: u64,
    commit_timestamp_capture_failures: u64,
    unique_committed_keys: usize,
    committed_keys: Vec<String>,
    committed_at_unix_ms: Vec<u64>,
    worker_stats: Vec<SqlWorkloadWorkerStats>,
    worker_errors: Vec<String>,
}

#[derive(serde::Serialize)]
struct SqlWorkloadSpecSummary {
    worker_count: usize,
    run_interval_ms: u64,
    table_name: String,
}

#[derive(serde::Serialize)]
struct StressScenarioSummary {
    schema_version: u32,
    scenario: String,
    status: String,
    started_at_unix_ms: u64,
    finished_at_unix_ms: u64,
    bootstrap_primary: Option<String>,
    final_primary: Option<String>,
    former_primary_demoted: Option<bool>,
    workload_spec: SqlWorkloadSpecSummary,
    workload: SqlWorkloadStats,
    ha_observations: HaObservationStats,
    notes: Vec<String>,
}

impl StressScenarioSummary {
    fn failed(scenario: &str, failure: String) -> Self {
        Self {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario.to_string(),
            status: "failed".to_string(),
            started_at_unix_ms: 0,
            finished_at_unix_ms: 0,
            bootstrap_primary: None,
            final_primary: None,
            former_primary_demoted: None,
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: 0,
                run_interval_ms: 0,
                table_name: String::new(),
            },
            workload: SqlWorkloadStats::default(),
            ha_observations: HaObservationStats::default(),
            notes: vec![failure],
        }
    }
}

#[derive(Clone, Copy)]
enum SqlErrorClass {
    Transient,
    Fencing,
    Hard,
}

fn classify_sql_error(message: &str) -> SqlErrorClass {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("read-only")
        || normalized.contains("read only")
        || normalized.contains("recovery is in progress")
        || normalized.contains("cannot execute insert")
    {
        return SqlErrorClass::Fencing;
    }
    if normalized.contains("connection refused")
        || normalized.contains("could not connect")
        || normalized.contains("connection reset")
        || normalized.contains("server closed the connection")
        || normalized.contains("timed out")
        || normalized.contains("timeout")
        || normalized.contains("the database system is starting up")
        || normalized.contains("the database system is shutting down")
        || normalized.contains("no route to host")
        || normalized.contains("broken pipe")
        || normalized.contains("does not exist")
        || normalized.contains("not yet accepting connections")
    {
        return SqlErrorClass::Transient;
    }
    if normalized.contains("syntax error")
        || normalized.contains("permission denied")
        || normalized.contains("invalid input syntax")
        || normalized.contains("unterminated quoted string")
    {
        return SqlErrorClass::Hard;
    }
    SqlErrorClass::Transient
}

fn sanitize_component(raw: &str) -> String {
    let mut safe: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if safe.is_empty() {
        safe = "unknown".to_string();
    }
    safe
}

fn sanitize_sql_identifier(raw: &str) -> String {
    let mut value: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if value.is_empty() {
        value = "ha_stress_table".to_string();
    }
    let first_is_alpha = value
        .chars()
        .next()
        .map(|ch| ch.is_ascii_alphabetic())
        .unwrap_or(false);
    if !first_is_alpha {
        value = format!("ha_stress_{value}");
    }
    value
}

fn sample_key_set(keys: &BTreeSet<String>) -> String {
    keys.iter().take(5).cloned().collect::<Vec<_>>().join(",")
}

fn committed_key_set_through_cutoff(
    workload: &SqlWorkloadStats,
    cutoff_ms: u64,
) -> Result<BTreeSet<String>, WorkerError> {
    let mut required_keys = BTreeSet::new();
    for worker in &workload.worker_stats {
        if worker.committed_keys.len() != worker.committed_at_unix_ms.len() {
            return Err(WorkerError::Message(format!(
                "worker {} committed key/timestamp length mismatch: keys={} timestamps={}",
                worker.worker_id,
                worker.committed_keys.len(),
                worker.committed_at_unix_ms.len()
            )));
        }
        for (key, committed_at_ms) in worker
            .committed_keys
            .iter()
            .zip(worker.committed_at_unix_ms.iter())
        {
            if *committed_at_ms <= cutoff_ms {
                required_keys.insert(key.clone());
            }
        }
    }
    Ok(required_keys)
}

fn assert_recovered_committed_keys_match_bounds(
    observed_rows: &[String],
    required_keys: &BTreeSet<String>,
    allowed_keys: &BTreeSet<String>,
    node_id: &str,
    table_name: &str,
) -> Result<u64, WorkerError> {
    let observed_row_count = u64::try_from(observed_rows.len()).map_err(|_| {
        WorkerError::Message(format!(
            "observed row count overflow while verifying {table_name} on {node_id}"
        ))
    })?;
    let observed_key_set: BTreeSet<String> = observed_rows.iter().cloned().collect();
    let observed_unique_count = u64::try_from(observed_key_set.len()).map_err(|_| {
        WorkerError::Message(format!(
            "observed unique key count overflow while verifying {table_name} on {node_id}"
        ))
    })?;
    if observed_unique_count != observed_row_count {
        return Err(WorkerError::Message(format!(
            "duplicate (worker_id,seq) rows detected on {node_id} for {table_name}: observed_rows={observed_row_count} unique_keys={observed_unique_count}"
        )));
    }

    let missing_keys: BTreeSet<String> = required_keys
        .difference(&observed_key_set)
        .cloned()
        .collect();
    let unexpected_keys: BTreeSet<String> =
        observed_key_set.difference(allowed_keys).cloned().collect();
    if !missing_keys.is_empty() || !unexpected_keys.is_empty() {
        return Err(WorkerError::Message(format!(
            "recovered key-set mismatch on {node_id} for {table_name}: missing_required_count={} missing_sample=[{}] unexpected_count={} unexpected_sample=[{}]",
            missing_keys.len(),
            sample_key_set(&missing_keys),
            unexpected_keys.len(),
            sample_key_set(&unexpected_keys),
        )));
    }

    Ok(observed_row_count)
}

impl ClusterFixture {
    async fn start(node_count: usize) -> Result<Self, WorkerError> {
        Self::start_with_config(ha_e2e::TestConfig {
            test_name: "ha-e2e-multi-node".to_string(),
            cluster_name: "cluster-e2e".to_string(),
            scope: "scope-ha-e2e".to_string(),
            node_count,
            etcd_members: vec![
                "etcd-a".to_string(),
                "etcd-b".to_string(),
                "etcd-c".to_string(),
            ],
            postgres_roles: None,
            mode: ha_e2e::Mode::Plain,
            timeouts: ha_e2e::TimeoutConfig {
                command_timeout: E2E_COMMAND_TIMEOUT,
                command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
                http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
                api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
                bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
                scenario_timeout: E2E_SCENARIO_TIMEOUT,
            },
        })
        .await
    }

    async fn start_with_config(config: ha_e2e::TestConfig) -> Result<Self, WorkerError> {
        let handle = ha_e2e::start_cluster(config).await?;

        let TestClusterHandle {
            guard,
            timeouts: _,
            binaries,
            superuser_username,
            superuser_dbname,
            etcd,
            nodes,
            tasks,
            etcd_proxies: _,
            api_proxies: _,
            pg_proxies: _,
        } = handle;

        Ok(Self {
            _guard: guard,
            pg_ctl_bin: binaries.pg_ctl.clone(),
            psql_bin: binaries.psql.clone(),
            superuser_username,
            superuser_dbname,
            etcd,
            nodes,
            tasks,
            timeline: Vec::new(),
        })
    }

    fn record(&mut self, message: impl Into<String>) {
        let stamp = match ha_e2e::util::unix_now() {
            Ok(value) => value.0.to_string(),
            Err(err) => format!("time_error:{err}"),
        };
        self.timeline.push(format!("[{stamp}] {}", message.into()));
    }

    fn node_by_id(&self, id: &str) -> Option<&ha_e2e::NodeHandle> {
        self.nodes.iter().find(|node| node.id == id)
    }

    fn node_index_by_id(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    fn postgres_port_by_id(&self, id: &str) -> Result<u16, WorkerError> {
        let node = self.node_by_id(id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for postgres port lookup: {id}"))
        })?;
        Ok(node.pg_port)
    }

    async fn run_sql_on_node(
        &self,
        node_id: &str,
        sql: &str,
        command_timeout: Duration,
    ) -> Result<String, WorkerError> {
        let port = self.postgres_port_by_id(node_id)?;
        ha_e2e::util::run_psql_statement(
            self.psql_bin.as_path(),
            port,
            self.superuser_username.as_str(),
            self.superuser_dbname.as_str(),
            sql,
            command_timeout,
            E2E_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
    }

    async fn run_sql_on_node_with_retry(
        &self,
        node_id: &str,
        sql: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match self
                .run_sql_on_node(node_id, sql, E2E_COMMAND_TIMEOUT)
                .await
            {
                Ok(output) => return Ok(output),
                Err(err) => {
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out running SQL on {node_id}; last_error={err}"
                        )));
                    }
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
                }
            }
        }
    }

    async fn cluster_sql_roles_best_effort(
        &self,
    ) -> Result<(Vec<(String, String)>, Vec<String>), WorkerError> {
        self.cluster_sql_roles_best_effort_with_timeout(E2E_COMMAND_TIMEOUT)
            .await
    }

    async fn cluster_sql_roles_best_effort_with_timeout(
        &self,
        command_timeout: Duration,
    ) -> Result<(Vec<(String, String)>, Vec<String>), WorkerError> {
        let mut roles = Vec::new();
        let mut errors = Vec::new();

        for node in &self.nodes {
            match self
                .run_sql_on_node(
                    node.id.as_str(),
                    "SELECT CASE WHEN pg_is_in_recovery() THEN 'replica' ELSE 'primary' END",
                    command_timeout,
                )
                .await
            {
                Ok(output) => {
                    let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                    let role = rows
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    roles.push((node.id.clone(), role));
                }
                Err(err) => {
                    errors.push(format!("node={} error={err}", node.id));
                }
            }
        }

        Ok((roles, errors))
    }

    async fn wait_for_rows_on_node(
        &self,
        node_id: &str,
        sql: &str,
        expected_rows: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let observation = match self
                .run_sql_on_node(node_id, sql, E2E_COMMAND_TIMEOUT)
                .await
            {
                Ok(output) => {
                    let rows = ha_e2e::util::parse_psql_rows(output.as_str());
                    if rows == expected_rows {
                        return Ok(());
                    }
                    format!("rows={rows:?}")
                }
                Err(err) => err.to_string(),
            };

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for expected rows on {node_id}; expected={expected_rows:?}; last_observation={observation}"
                )));
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    fn sql_workload_ctx(&self, spec: &SqlWorkloadSpec) -> Result<SqlWorkloadCtx, WorkerError> {
        if spec.worker_count == 0 {
            return Err(WorkerError::Message(
                "sql workload requires at least one worker".to_string(),
            ));
        }
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "sql workload cannot start: cluster has no nodes".to_string(),
            ));
        }
        let targets = self
            .nodes
            .iter()
            .map(|node| SqlWorkloadTarget {
                node_id: node.id.clone(),
                port: node.pg_port,
            })
            .collect::<Vec<_>>();
        Ok(SqlWorkloadCtx {
            psql_bin: self.psql_bin.clone(),
            superuser_username: self.superuser_username.clone(),
            superuser_dbname: self.superuser_dbname.clone(),
            scenario_name: spec.scenario_name.clone(),
            table_name: sanitize_sql_identifier(spec.table_name.as_str()),
            interval: spec.interval(),
            targets,
        })
    }

    async fn prepare_stress_table(
        &self,
        bootstrap_primary: &str,
        table_name: &str,
    ) -> Result<(), WorkerError> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (worker_id INTEGER NOT NULL, seq BIGINT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY (worker_id, seq))"
        );
        self.run_sql_on_node_with_retry(bootstrap_primary, sql.as_str(), Duration::from_secs(30))
            .await?;
        Ok(())
    }

    async fn start_sql_workload(
        &mut self,
        spec: SqlWorkloadSpec,
    ) -> Result<SqlWorkloadHandle, WorkerError> {
        let workload_ctx = self.sql_workload_ctx(&spec)?;
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
        let mut joins = Vec::with_capacity(spec.worker_count);
        for worker_id in 0..spec.worker_count {
            let worker_ctx = workload_ctx.clone();
            let worker_stop_rx = stop_rx.clone();
            joins.push(tokio::spawn(async move {
                run_sql_workload_worker(worker_ctx, worker_id, worker_stop_rx).await
            }));
        }
        self.record(format!(
            "sql workload started: scenario={} table={} workers={} interval_ms={}",
            spec.scenario_name, workload_ctx.table_name, spec.worker_count, spec.run_interval_ms
        ));
        Ok(SqlWorkloadHandle {
            spec,
            started_at_unix_ms,
            stop_tx,
            joins,
        })
    }

    async fn stop_sql_workload_and_collect(
        &mut self,
        handle: SqlWorkloadHandle,
        drain: Duration,
    ) -> Result<SqlWorkloadStats, WorkerError> {
        let SqlWorkloadHandle {
            spec,
            started_at_unix_ms,
            stop_tx,
            joins,
        } = handle;
        let _ = stop_tx.send(true);
        tokio::time::sleep(drain).await;

        let mut stats = SqlWorkloadStats {
            scenario_name: spec.scenario_name.clone(),
            table_name: sanitize_sql_identifier(spec.table_name.as_str()),
            worker_count: spec.worker_count,
            started_at_unix_ms,
            ..SqlWorkloadStats::default()
        };
        let mut committed_key_set: BTreeSet<String> = BTreeSet::new();
        for join in joins {
            match join.await {
                Ok(Ok(worker)) => {
                    stats.attempted_writes = stats
                        .attempted_writes
                        .saturating_add(worker.attempted_writes);
                    stats.committed_writes = stats
                        .committed_writes
                        .saturating_add(worker.committed_writes);
                    stats.attempted_reads =
                        stats.attempted_reads.saturating_add(worker.attempted_reads);
                    stats.read_successes =
                        stats.read_successes.saturating_add(worker.read_successes);
                    stats.transient_failures = stats
                        .transient_failures
                        .saturating_add(worker.transient_failures);
                    stats.fencing_failures = stats
                        .fencing_failures
                        .saturating_add(worker.fencing_failures);
                    stats.hard_failures = stats.hard_failures.saturating_add(worker.hard_failures);
                    stats.commit_timestamp_capture_failures = stats
                        .commit_timestamp_capture_failures
                        .saturating_add(worker.commit_timestamp_capture_failures);
                    stats
                        .committed_at_unix_ms
                        .extend(worker.committed_at_unix_ms.iter().copied());
                    for key in &worker.committed_keys {
                        committed_key_set.insert(key.clone());
                    }
                    stats.worker_stats.push(worker);
                }
                Ok(Err(err)) => {
                    stats.worker_errors.push(err.to_string());
                }
                Err(err) => {
                    stats
                        .worker_errors
                        .push(format!("workload worker join failed: {err}"));
                }
            }
        }
        let worker_error_count_u64 = u64::try_from(stats.worker_errors.len()).unwrap_or(u64::MAX);
        stats.hard_failures = stats.hard_failures.saturating_add(worker_error_count_u64);
        stats.committed_keys = committed_key_set.into_iter().collect();
        stats.unique_committed_keys = stats.committed_keys.len();
        stats.finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        stats.duration_ms = stats
            .finished_at_unix_ms
            .saturating_sub(stats.started_at_unix_ms);
        self.record(format!(
            "sql workload stopped: scenario={} committed={} unique_keys={} transient={} fencing={} hard={}",
            stats.scenario_name,
            stats.committed_writes,
            stats.unique_committed_keys,
            stats.transient_failures,
            stats.fencing_failures,
            stats.hard_failures
        ));
        Ok(stats)
    }

    async fn sample_ha_states_window(
        &mut self,
        window: Duration,
        interval: Duration,
        ring_capacity: usize,
    ) -> Result<HaObservationStats, WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity,
        });
        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self
                .poll_node_ha_states_best_effort_with_timeout(Duration::from_secs(8))
                .await
            {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut errors = Vec::new();
                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => states.push(state),
                            Err(err) => {
                                errors.push(format!("node={node_id} error={err}"));
                            }
                        }
                    }

                    observer.record_api_states(&states, &errors)?;
                }
                Err(err) => {
                    observer.record_transport_error(err.to_string());
                }
            };
            if tokio::time::Instant::now() >= deadline {
                return Ok(observer.into_stats());
            }
            tokio::time::sleep(interval).await;
        }
    }

    fn count_commits_after_cutoff_strict(
        workload: &SqlWorkloadStats,
        cutoff_ms: u64,
    ) -> Result<usize, WorkerError> {
        if workload.commit_timestamp_capture_failures > 0 {
            return Err(WorkerError::Message(format!(
                "cannot evaluate fencing cutoff: commit_timestamp_capture_failures={}",
                workload.commit_timestamp_capture_failures
            )));
        }
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "cannot evaluate fencing cutoff with zero committed writes".to_string(),
            ));
        }
        let committed_writes_usize = usize::try_from(workload.committed_writes).map_err(|_| {
            WorkerError::Message("committed_writes does not fit into usize".to_string())
        })?;
        if workload.committed_at_unix_ms.len() != committed_writes_usize {
            return Err(WorkerError::Message(format!(
                "cannot evaluate fencing cutoff: committed_at_unix_ms incomplete (timestamps={} committed_writes={})",
                workload.committed_at_unix_ms.len(),
                workload.committed_writes
            )));
        }
        if workload.committed_at_unix_ms.contains(&0) {
            return Err(WorkerError::Message(
                "cannot evaluate fencing cutoff: committed_at_unix_ms contains 0 timestamp"
                    .to_string(),
            ));
        }

        Ok(workload
            .committed_at_unix_ms
            .iter()
            .filter(|timestamp| **timestamp > cutoff_ms)
            .count())
    }

    async fn assert_former_primary_demoted_or_unreachable_after_transition(
        &mut self,
        former_primary: &str,
    ) -> Result<(), WorkerError> {
        let node_index = self.node_index_by_id(former_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "unknown former primary for demotion assertion: {former_primary}"
            ))
        })?;
        match self.fetch_node_ha_state_by_index(node_index).await {
            Ok(state) => {
                if state.ha_phase == "Primary" {
                    return Err(WorkerError::Message(format!(
                        "former primary {former_primary} still reports Primary phase"
                    )));
                }
                Ok(())
            }
            Err(err) => {
                self.record(format!(
                    "former primary {former_primary} API remained unreachable after transition; treating unreachable API as demotion evidence: {err}"
                ));
                Ok(())
            }
        }
    }

    async fn assert_table_key_integrity_on_node(
        &self,
        node_id: &str,
        table_name: &str,
        min_rows: u64,
        timeout: Duration,
    ) -> Result<u64, WorkerError> {
        let port = self.postgres_port_by_id(node_id)?;
        let count_sql = format!("SELECT COUNT(*)::bigint FROM {table_name}");
        let duplicate_sql = format!(
            "SELECT COUNT(*)::bigint FROM (SELECT worker_id, seq, COUNT(*) AS c FROM {table_name} GROUP BY worker_id, seq HAVING COUNT(*) > 1) d"
        );
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let count_raw = match ha_e2e::util::run_psql_statement(
                self.psql_bin.as_path(),
                port,
                self.superuser_username.as_str(),
                self.superuser_dbname.as_str(),
                count_sql.as_str(),
                E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
                E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
            )
            .await
            {
                Ok(value) => value,
                Err(err) => {
                    let detail = format!("row count query failed: {err}");
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out verifying table integrity on {node_id}; last_observation={detail}"
                        )));
                    }
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
                    continue;
                }
            };
            let duplicate_raw = match ha_e2e::util::run_psql_statement(
                self.psql_bin.as_path(),
                port,
                self.superuser_username.as_str(),
                self.superuser_dbname.as_str(),
                duplicate_sql.as_str(),
                E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
                E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
            )
            .await
            {
                Ok(value) => value,
                Err(err) => {
                    let detail = format!("duplicate query failed: {err}");
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out verifying table integrity on {node_id}; last_observation={detail}"
                        )));
                    }
                    tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
                    continue;
                }
            };
            let row_count = ha_e2e::util::parse_single_u64(count_raw.as_str())?;
            let duplicate_count = ha_e2e::util::parse_single_u64(duplicate_raw.as_str())?;
            if duplicate_count > 0 {
                return Err(WorkerError::Message(format!(
                    "duplicate (worker_id,seq) rows detected on {node_id}: {duplicate_count}"
                )));
            }
            if row_count >= min_rows {
                return Ok(row_count);
            }
            let detail = format!("row_count={row_count} below min_rows={min_rows}");
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out verifying table integrity on {node_id}; last_observation={detail}"
                )));
            }
            tokio::time::sleep(E2E_SQL_RETRY_INTERVAL).await;
        }
    }

    async fn assert_table_key_integrity_strict(
        &mut self,
        preferred_node_id: &str,
        table_name: &str,
        min_rows: u64,
        per_node_timeout: Duration,
    ) -> Result<(String, u64), WorkerError> {
        let mut node_ids = Vec::new();
        if self.node_by_id(preferred_node_id).is_some() {
            node_ids.push(preferred_node_id.to_string());
        }
        for node in &self.nodes {
            if node.id != preferred_node_id {
                node_ids.push(node.id.clone());
            }
        }

        if node_ids.is_empty() {
            return Err(WorkerError::Message(format!(
                "cannot verify table integrity: no nodes available for {table_name}"
            )));
        }

        let mut errors = Vec::new();
        for node_id in node_ids {
            match self
                .assert_table_key_integrity_on_node(
                    node_id.as_str(),
                    table_name,
                    min_rows,
                    per_node_timeout,
                )
                .await
            {
                Ok(row_count) => return Ok((node_id, row_count)),
                Err(err) => {
                    let message = err.to_string();
                    // Duplicate rows / empty table are hard failures when a node is reachable enough
                    // to answer queries (this indicates a real integrity problem).
                    if message.contains("duplicate (worker_id,seq) rows detected")
                        || message.contains("below min_rows")
                    {
                        return Err(err);
                    }
                    errors.push(format!("{node_id}: {message}"));
                }
            }
        }

        Err(WorkerError::Message(format!(
            "table integrity could not be verified on any node for {table_name}; errors={errors:?}"
        )))
    }

    async fn assert_table_recovery_key_integrity_on_node(
        &mut self,
        node_id: &str,
        table_name: &str,
        required_keys: &BTreeSet<String>,
        allowed_keys: &BTreeSet<String>,
        timeout: Duration,
    ) -> Result<u64, WorkerError> {
        let query = format!(
            "SELECT worker_id::text || ':' || seq::text FROM {table_name} ORDER BY worker_id, seq"
        );
        let rows_raw = self
            .run_sql_on_node_with_retry(node_id, query.as_str(), timeout)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "recovery key verification query failed on {node_id} for {table_name}: {err}"
                ))
            })?;
        let observed_rows = ha_e2e::util::parse_psql_rows(rows_raw.as_str());
        assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            required_keys,
            allowed_keys,
            node_id,
            table_name,
        )
    }

    fn assert_no_split_brain_write_evidence(
        workload: &SqlWorkloadStats,
        _ha_stats: &HaObservationStats,
    ) -> Result<(), WorkerError> {
        if workload.unique_committed_keys
            != usize::try_from(workload.committed_writes).unwrap_or(usize::MAX)
        {
            return Err(WorkerError::Message(format!(
                "duplicate committed write keys detected: committed_writes={} unique_keys={}",
                workload.committed_writes, workload.unique_committed_keys
            )));
        }
        if workload.hard_failures > 0 {
            return Err(WorkerError::Message(format!(
                "hard SQL failures detected during stress workload: hard_failures={} worker_errors={:?}",
                workload.hard_failures, workload.worker_errors
            )));
        }
        Ok(())
    }

    fn update_phase_history(
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
        states: &[HaStateResponse],
    ) {
        for state in states {
            phase_history
                .entry(state.self_member_id.clone())
                .or_default()
                .insert(state.ha_phase.to_string());
        }
    }

    fn format_phase_history(phase_history: &BTreeMap<String, BTreeSet<String>>) -> String {
        let mut node_entries = Vec::with_capacity(phase_history.len());
        for (node_id, phases) in phase_history {
            let phase_list: Vec<&str> = phases.iter().map(String::as_str).collect();
            node_entries.push(format!("{node_id}:{}", phase_list.join("|")));
        }
        node_entries.join(",")
    }

    async fn wait_for_stable_primary(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = "none".to_string();
        let mut last_candidate: Option<String> = None;
        let mut last_state_summary: Option<String> = None;
        let mut stable_count = 0usize;

        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    Self::update_phase_history(phase_history, states.as_slice());
                    let state_summary = states
                        .iter()
                        .map(|state| {
                            let leader = state.leader.as_deref().unwrap_or("none");
                            format!(
                                "{}:{}:leader={}",
                                state.self_member_id, state.ha_phase, leader
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    if last_state_summary
                        .as_deref()
                        .map(|prior| prior != state_summary.as_str())
                        .unwrap_or(true)
                    {
                        self.record(format!("stable-primary poll states: {state_summary}"));
                        last_state_summary = Some(state_summary);
                    }
                    let primaries = Self::primary_members(states.as_slice());
                    if primaries.len() == 1 {
                        if let Some(primary) = primaries.into_iter().next() {
                            let excluded = excluded_primary
                                .map(|excluded_id| excluded_id == primary)
                                .unwrap_or(false);
                            if !excluded {
                                if last_candidate.as_deref() == Some(primary.as_str()) {
                                    stable_count = stable_count.saturating_add(1);
                                } else {
                                    stable_count = 1;
                                    last_candidate = Some(primary.clone());
                                }
                                if stable_count >= required_consecutive {
                                    return Ok(primary);
                                }
                            } else {
                                stable_count = 0;
                                last_candidate = None;
                            }
                        }
                    } else {
                        stable_count = 0;
                        last_candidate = None;
                    }
                }
                Err(err) => {
                    stable_count = 0;
                    last_candidate = None;
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary via API; last_error={last_error}"
                )));
            }
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
    }

    async fn wait_for_stable_primary_best_effort(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
        min_observed_nodes: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }
        if min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = "none".to_string();
        let mut last_candidate: Option<String> = None;
        let mut last_state_summary: Option<String> = None;
        let mut stable_count = 0usize;

        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut fragments = Vec::with_capacity(polled.len());

                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => {
                                let leader = state.leader.as_deref().unwrap_or("none");
                                fragments.push(format!(
                                    "{}:{}:leader={leader}",
                                    state.self_member_id, state.ha_phase
                                ));
                                states.push(state);
                            }
                            Err(err) => {
                                fragments.push(format!("{node_id}:error={err}"));
                                last_error = format!("HA state poll failed for {node_id}: {err}");
                            }
                        }
                    }

                    let state_summary = fragments.join(", ");
                    if last_state_summary
                        .as_deref()
                        .map(|prior| prior != state_summary.as_str())
                        .unwrap_or(true)
                    {
                        self.record(format!(
                            "stable-primary best-effort poll states: {state_summary}"
                        ));
                        last_state_summary = Some(state_summary);
                    }

                    if states.len() < min_observed_nodes {
                        stable_count = 0;
                        last_candidate = None;
                        last_error = format!(
                            "insufficient observed HA states: observed={} required={min_observed_nodes}",
                            states.len()
                        );
                    } else {
                        Self::update_phase_history(phase_history, states.as_slice());
                        let primaries = Self::primary_members(states.as_slice());
                        if primaries.len() == 1 {
                            if let Some(primary) = primaries.into_iter().next() {
                                let excluded = excluded_primary
                                    .map(|excluded_id| excluded_id == primary)
                                    .unwrap_or(false);
                                if !excluded {
                                    if last_candidate.as_deref() == Some(primary.as_str()) {
                                        stable_count = stable_count.saturating_add(1);
                                    } else {
                                        stable_count = 1;
                                        last_candidate = Some(primary.clone());
                                    }
                                    if stable_count >= required_consecutive {
                                        return Ok(primary);
                                    }
                                } else {
                                    stable_count = 0;
                                    last_candidate = None;
                                }
                            }
                        } else {
                            stable_count = 0;
                            last_candidate = None;
                        }
                    }
                }
                Err(err) => {
                    stable_count = 0;
                    last_candidate = None;
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary via best-effort API polling; last_error={last_error}"
                )));
            }
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
    }

    fn assert_phase_history_contains_failover(
        phase_history: &BTreeMap<String, BTreeSet<String>>,
        former_primary: &str,
        new_primary: &str,
    ) -> Result<(), WorkerError> {
        const PRIMARY_PHASE: &str = "primary";

        let former_phases = phase_history.get(former_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing phase history for former primary {former_primary}"
            ))
        })?;
        if !former_phases.contains(PRIMARY_PHASE) {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} never observed in Primary phase"
            )));
        }
        if !former_phases.iter().any(|phase| phase != PRIMARY_PHASE) {
            return Err(WorkerError::Message(format!(
                "former primary {former_primary} never observed leaving Primary phase"
            )));
        }

        let promoted_phases = phase_history.get(new_primary).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing phase history for promoted primary {new_primary}"
            ))
        })?;
        if !promoted_phases.contains(PRIMARY_PHASE) {
            return Err(WorkerError::Message(format!(
                "new primary {new_primary} never observed in Primary phase"
            )));
        }

        Ok(())
    }

    fn node_api_base_url_by_index(
        &self,
        node_index: usize,
    ) -> Result<(String, String), WorkerError> {
        let node = self.nodes.get(node_index).ok_or_else(|| {
            WorkerError::Message(format!("invalid node index for API request: {node_index}"))
        })?;
        Ok((node.id.clone(), format!("http://{}", node.api_observe_addr)))
    }

    fn cli_api_client_for_node_index(
        &self,
        node_index: usize,
    ) -> Result<(String, CliApiClient), WorkerError> {
        let (node_id, base_url) = self.node_api_base_url_by_index(node_index)?;
        let timeout_ms = e2e_http_timeout_ms()?;
        let client = CliApiClient::new(base_url, timeout_ms, None, None)
            .map_err(|err| WorkerError::Message(format!("build CliApiClient failed: {err}")))?;
        Ok((node_id, client))
    }

    async fn request_switchover_via_cli(&mut self, requested_by: &str) -> Result<(), WorkerError> {
        if self.nodes.is_empty() {
            return Err(WorkerError::Message(
                "no nodes available for API control".to_string(),
            ));
        }

        let timeout_ms = e2e_http_timeout_ms()?;

        // Any node API can write the switchover intent. Iterate across all node APIs because the
        // former primary can be transiently unavailable while replicas are still healthy enough to
        // accept the operator request.
        let max_transport_rounds: usize = 5;
        let mut last_transport_error = "transport error".to_string();
        let mut output: Option<String> = None;

        for round in 1..=max_transport_rounds {
            for node_index in 0..self.nodes.len() {
                let (node_id, base_url) = self.node_api_base_url_by_index(node_index)?;
                self.record(format!(
                    "cli request start: round={round}/{max_transport_rounds} node={node_id} ha switchover request requested_by={requested_by}"
                ));
                let argv: Vec<String> = vec![
                    "pgtuskmasterctl".to_string(),
                    "--base-url".to_string(),
                    base_url,
                    "--timeout-ms".to_string(),
                    timeout_ms.to_string(),
                    "--output".to_string(),
                    "json".to_string(),
                    "ha".to_string(),
                    "switchover".to_string(),
                    "request".to_string(),
                    "--requested-by".to_string(),
                    requested_by.to_string(),
                ];
                let cli = Cli::try_parse_from(argv).map_err(|err| {
                    WorkerError::Message(format!("parse switchover CLI args failed: {err}"))
                })?;
                match cli::run(cli).await {
                    Ok(out) => {
                        self.record(format!(
                            "cli request success: round={round}/{max_transport_rounds} node={node_id} ha switchover request accepted=true requested_by={requested_by}"
                        ));
                        output = Some(out);
                        break;
                    }
                    Err(err) => match err {
                        CliError::Transport(_) => {
                            let err_string = err.to_string();
                            last_transport_error =
                                format!("node={node_id} round={round} err={err_string}");
                            self.record(format!(
                                "cli request transport failure: round={round}/{max_transport_rounds} node={node_id} requested_by={requested_by} err={err_string}"
                            ));
                        }
                        _ => {
                            return Err(WorkerError::Message(format!(
                                "run switchover CLI command failed via {node_id}: {err}"
                            )));
                        }
                    },
                }
            }

            if output.is_some() {
                break;
            }

            if round < max_transport_rounds {
                let backoff_ms = 200_u64.saturating_mul(round as u64);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        let output = match output {
            Some(out) => out,
            None => {
                return Err(WorkerError::Message(format!(
                    "run switchover CLI command failed after {max_transport_rounds} round(s) across {} node(s): {last_transport_error}",
                    self.nodes.len()
                )));
            }
        };

        let accepted =
            serde_json::from_str::<CliAcceptedResponse>(output.as_str()).map_err(|err| {
                WorkerError::Message(format!(
                    "decode switchover CLI response failed: {err}; output={}",
                    output.trim()
                ))
            })?;
        if !accepted.accepted {
            return Err(WorkerError::Message(
                "switchover CLI response returned accepted=false".to_string(),
            ));
        }
        Ok(())
    }

    async fn request_switchover_until_stable_primary_changes(
        &mut self,
        previous_primary: &str,
        requested_by: &str,
        max_attempts: usize,
        per_attempt_timeout: Duration,
        required_consecutive: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if max_attempts == 0 {
            return Err(WorkerError::Message(
                "switchover attempts must be greater than zero".to_string(),
            ));
        }
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }

        let mut last_error = "none".to_string();
        for attempt in 1..=max_attempts {
            self.request_switchover_via_cli(requested_by).await?;
            match self
                .wait_for_stable_primary_best_effort(
                    per_attempt_timeout,
                    Some(previous_primary),
                    required_consecutive,
                    1,
                    phase_history,
                )
                .await
            {
                Ok(primary) => return Ok(primary),
                Err(err) => {
                    let stable_wait_error = err.to_string();
                    self.record(format!(
                        "switchover attempt {attempt}/{max_attempts} stable-primary wait failed after accepted request: {stable_wait_error}; retrying with relaxed primary-change detection"
                    ));
                    match self
                        .wait_for_primary_change_best_effort(
                            per_attempt_timeout,
                            previous_primary,
                            1,
                            phase_history,
                        )
                        .await
                    {
                        Ok(primary) => return Ok(primary),
                        Err(change_err) => {
                            last_error = format!(
                                "{stable_wait_error}; fallback primary-change detection failed: {change_err}"
                            );
                            self.record(format!(
                                "switchover attempt {attempt}/{max_attempts} did not change primary from {previous_primary}: {last_error}"
                            ));
                        }
                    }
                }
            }
            if attempt < max_attempts {
                tokio::time::sleep(E2E_SWITCHOVER_RETRY_BACKOFF).await;
            }
        }

        Err(WorkerError::Message(format!(
            "switchover did not change primary from {previous_primary} after {max_attempts} attempt(s); last_error={last_error}"
        )))
    }

    // /ha/state polling is the canonical post-start observation path.
    async fn fetch_node_ha_state_by_index(
        &mut self,
        node_index: usize,
    ) -> Result<HaStateResponse, WorkerError> {
        let node_addr = self
            .nodes
            .get(node_index)
            .ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid node index for HA state fetch: {node_index}"
                ))
            })?
            .api_observe_addr;
        let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
        ha_e2e::util::get_ha_state_with_fallback(
            &client,
            node_id.as_str(),
            node_addr,
            E2E_HTTP_STEP_TIMEOUT,
        )
        .await
    }

    async fn poll_node_ha_states_best_effort(
        &self,
    ) -> Result<Vec<(String, Result<HaStateResponse, WorkerError>)>, WorkerError> {
        self.poll_node_ha_states_best_effort_with_timeout(E2E_HTTP_STEP_TIMEOUT)
            .await
    }

    async fn poll_node_ha_states_best_effort_with_timeout(
        &self,
        http_step_timeout: Duration,
    ) -> Result<Vec<(String, Result<HaStateResponse, WorkerError>)>, WorkerError> {
        let node_count = self.nodes.len();
        let mut joins = Vec::with_capacity(node_count);

        for node_index in 0..node_count {
            let node = self.nodes.get(node_index).ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid node index for HA state poll: {node_index}"
                ))
            })?;
            let (node_id, client) = self.cli_api_client_for_node_index(node_index)?;
            let node_addr = node.api_addr;
            joins.push(tokio::task::spawn_local(async move {
                let result = ha_e2e::util::get_ha_state_with_fallback(
                    &client,
                    node_id.as_str(),
                    node_addr,
                    http_step_timeout,
                )
                .await;
                (node_id, result)
            }));
        }

        let mut results = Vec::with_capacity(node_count);
        for join in joins {
            let joined = join
                .await
                .map_err(|err| WorkerError::Message(format!("HA state poll join failed: {err}")))?;
            results.push(joined);
        }

        Ok(results)
    }

    async fn cluster_ha_states(&mut self) -> Result<Vec<HaStateResponse>, WorkerError> {
        self.ensure_runtime_tasks_healthy().await?;
        let polled = self.poll_node_ha_states_best_effort().await?;
        let mut states = Vec::with_capacity(polled.len());
        for (node_id, result) in polled {
            let state = result.map_err(|err| {
                WorkerError::Message(format!("HA state poll failed for {node_id}: {err}"))
            })?;
            states.push(state);
        }
        Ok(states)
    }

    async fn ensure_runtime_tasks_healthy(&mut self) -> Result<(), WorkerError> {
        let mut index = 0usize;
        while index < self.tasks.len() {
            if !self.tasks[index].is_finished() {
                index = index.saturating_add(1);
                continue;
            }

            let node_id = self
                .nodes
                .get(index)
                .map(|node| node.id.clone())
                .unwrap_or_else(|| format!("index-{index}"));
            let task = self.tasks.swap_remove(index);
            let joined = task
                .await
                .map_err(|err| WorkerError::Message(format!("runtime task join failed: {err}")))?;
            match joined {
                Ok(()) => {
                    return Err(WorkerError::Message(format!(
                        "runtime task for {node_id} exited unexpectedly"
                    )));
                }
                Err(err) => {
                    return Err(WorkerError::Message(format!(
                        "runtime task for {node_id} failed: {err}"
                    )));
                }
            }
        }
        Ok(())
    }

    fn primary_members(states: &[HaStateResponse]) -> Vec<String> {
        states
            .iter()
            .filter(|state| state.ha_phase == "Primary")
            .map(|state| state.self_member_id.clone())
            .collect()
    }

    async fn wait_for_primary_change(
        &mut self,
        previous: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error: Option<String> = None;
        loop {
            match self.cluster_ha_states().await {
                Ok(states) => {
                    let primaries = Self::primary_members(&states);
                    if primaries.len() == 1 {
                        if let Some(primary) = primaries.into_iter().next() {
                            if primary != previous {
                                return Ok(primary);
                            }
                        }
                    }
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                let detail = last_error
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for primary change from {previous} via API; last_error={detail}"
                )));
            }
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
    }

    async fn wait_for_primary_change_best_effort(
        &mut self,
        timeout: Duration,
        previous: &str,
        min_observed_nodes: usize,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut last_error = "none".to_string();
        let mut last_state_summary: Option<String> = None;

        loop {
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut fragments = Vec::with_capacity(polled.len());

                    for (node_id, state_result) in polled {
                        match state_result {
                            Ok(state) => {
                                let leader = state.leader.as_deref().unwrap_or("none");
                                fragments.push(format!(
                                    "{}:{}:leader={leader}",
                                    state.self_member_id, state.ha_phase
                                ));
                                states.push(state);
                            }
                            Err(err) => {
                                fragments.push(format!("{node_id}:error={err}"));
                                last_error = format!("HA state poll failed for {node_id}: {err}");
                            }
                        }
                    }

                    let state_summary = fragments.join(", ");
                    if last_state_summary
                        .as_deref()
                        .map(|prior| prior != state_summary.as_str())
                        .unwrap_or(true)
                    {
                        self.record(format!(
                            "primary-change best-effort poll states: {state_summary}"
                        ));
                        last_state_summary = Some(state_summary);
                    }

                    if states.len() < min_observed_nodes {
                        last_error = format!(
                            "insufficient observed HA states: observed={} required={min_observed_nodes}",
                            states.len()
                        );
                    } else {
                        Self::update_phase_history(phase_history, states.as_slice());
                        let primaries = Self::primary_members(states.as_slice());
                        if primaries.len() == 1 {
                            if let Some(primary) = primaries.into_iter().next() {
                                if primary != previous {
                                    return Ok(primary);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    last_error = err.to_string();
                }
            }

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for primary change from {previous} via best-effort API polling; last_error={last_error}"
                )));
            }
            tokio::time::sleep(E2E_STABLE_PRIMARY_API_POLL_INTERVAL).await;
        }
    }

    async fn wait_for_stable_primary_via_sql(
        &mut self,
        timeout: Duration,
        excluded_primary: Option<&str>,
        required_consecutive: usize,
        min_observed_nodes: usize,
    ) -> Result<String, WorkerError> {
        if required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }
        if min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let mut stable_count = 0usize;
        let mut last_candidate: Option<String> = None;
        let mut last_observation = "none".to_string();

        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for stable primary via SQL; excluded={excluded_primary:?}; last_observation={last_observation}"
                )));
            }

            let (sql_roles, sql_errors) = self.cluster_sql_roles_best_effort().await?;
            let observed_nodes = sql_roles.len();
            let primary_nodes = sql_roles
                .iter()
                .filter(|(_, role)| role == "primary")
                .map(|(node_id, _)| node_id.clone())
                .collect::<Vec<_>>();
            let role_fragments = sql_roles
                .iter()
                .map(|(node_id, role)| format!("{node_id}:{role}"))
                .collect::<Vec<_>>();
            let error_fragments = sql_errors
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" | ");

            last_observation = format!(
                "observed_nodes={observed_nodes} roles=[{}] errors={error_fragments}",
                role_fragments.join(", ")
            );

            if observed_nodes < min_observed_nodes {
                stable_count = 0;
                last_candidate = None;
            } else if primary_nodes.len() == 1 {
                let candidate = primary_nodes[0].clone();
                let excluded = excluded_primary
                    .map(|excluded_id| excluded_id == candidate)
                    .unwrap_or(false);
                if !excluded {
                    if last_candidate.as_deref() == Some(candidate.as_str()) {
                        stable_count = stable_count.saturating_add(1);
                    } else {
                        stable_count = 1;
                        last_candidate = Some(candidate.clone());
                    }
                    if stable_count >= required_consecutive {
                        self.record(format!(
                            "stable-primary SQL poll converged: {}",
                            role_fragments.join(", ")
                        ));
                        return Ok(candidate);
                    }
                } else {
                    stable_count = 0;
                    last_candidate = None;
                }
            } else {
                stable_count = 0;
                last_candidate = None;
            }

            tokio::time::sleep(E2E_STABLE_PRIMARY_SQL_POLL_INTERVAL).await;
        }
    }

    async fn wait_for_stable_primary_resilient(
        &mut self,
        plan: StablePrimaryWaitPlan<'_>,
        phase_history: &mut BTreeMap<String, BTreeSet<String>>,
    ) -> Result<String, WorkerError> {
        if plan.required_consecutive == 0 {
            return Err(WorkerError::Message(
                "required_consecutive must be greater than zero".to_string(),
            ));
        }
        if plan.fallback_required_consecutive == 0 {
            return Err(WorkerError::Message(
                "fallback_required_consecutive must be greater than zero".to_string(),
            ));
        }
        if plan.min_observed_nodes == 0 {
            return Err(WorkerError::Message(
                "min_observed_nodes must be greater than zero".to_string(),
            ));
        }

        let strict_timeout = std::cmp::min(plan.timeout, E2E_STABLE_PRIMARY_STRICT_TIMEOUT_CAP);
        let api_fallback_timeout = std::cmp::min(
            plan.fallback_timeout,
            E2E_STABLE_PRIMARY_API_FALLBACK_TIMEOUT_CAP,
        );
        let sql_fallback_timeout = std::cmp::min(
            plan.fallback_timeout,
            E2E_STABLE_PRIMARY_SQL_FALLBACK_TIMEOUT_CAP,
        );
        let strict_required_consecutive = plan
            .required_consecutive
            .min(E2E_STABLE_PRIMARY_STRICT_CONSECUTIVE_CAP);
        let relaxed_required_consecutive = plan
            .fallback_required_consecutive
            .min(E2E_STABLE_PRIMARY_RELAXED_CONSECUTIVE_CAP);

        match self
            .wait_for_stable_primary(
                strict_timeout,
                plan.excluded_primary,
                strict_required_consecutive,
                phase_history,
            )
            .await
        {
            Ok(primary) => Ok(primary),
            Err(wait_err) => {
                self.record(format!(
                    "{}: strict stable-primary wait failed: {wait_err}; retrying with best-effort API polling",
                    plan.context
                ));
                match self
                    .wait_for_stable_primary_best_effort(
                        api_fallback_timeout,
                        plan.excluded_primary,
                        relaxed_required_consecutive,
                        plan.min_observed_nodes,
                        phase_history,
                    )
                    .await
                {
                    Ok(primary) => Ok(primary),
                    Err(best_effort_err) => {
                        self.record(format!(
                            "{}: best-effort API stable-primary wait failed: {best_effort_err}; retrying with SQL role polling",
                            plan.context
                        ));
                        self.wait_for_stable_primary_via_sql(
                            sql_fallback_timeout,
                            plan.excluded_primary,
                            relaxed_required_consecutive,
                            plan.min_observed_nodes,
                        )
                        .await
                    }
                }
            }
        }
    }

    async fn assert_no_dual_primary_window(&mut self, window: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + window;
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 16,
        });
        loop {
            observer.record_poll_attempt();
            self.ensure_runtime_tasks_healthy().await?;
            match self.poll_node_ha_states_best_effort().await {
                Ok(polled) => {
                    let mut states = Vec::new();
                    let mut errors = Vec::new();
                    for (node_id, result) in polled {
                        match result {
                            Ok(state) => states.push(state),
                            Err(err) => errors.push(format!("node={node_id} error={err}")),
                        }
                    }

                    if states.is_empty() {
                        let (sql_roles, sql_errors) = self.cluster_sql_roles_best_effort().await?;
                        if sql_roles.is_empty() {
                            observer.record_observation_gap(&errors, &sql_errors);
                        } else {
                            observer.record_sql_roles(&sql_roles, &sql_errors)?;
                        }
                    } else {
                        observer.record_api_states(&states, &errors)?;
                    }
                }
                Err(err) => {
                    observer.record_transport_error(err.to_string());
                }
            }
            if tokio::time::Instant::now() >= deadline {
                return observer.finalize_no_dual_primary_window();
            }
            tokio::time::sleep(E2E_NO_DUAL_PRIMARY_SAMPLE_INTERVAL).await;
        }
    }

    async fn wait_for_all_nodes_failsafe(&mut self, timeout: Duration) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut observed_api_failsafe_nodes: BTreeSet<String> = BTreeSet::new();
        let mut observed_api_non_primary_nodes: BTreeSet<String> = BTreeSet::new();
        let mut last_observation: Option<String> = None;
        let mut last_recorded_at = tokio::time::Instant::now();
        let node_count = self.nodes.len();
        if node_count == 0 {
            return Err(WorkerError::Message(
                "cannot wait for fail-safe with zero nodes".to_string(),
            ));
        }

        loop {
            if tokio::time::Instant::now() >= deadline {
                let detail = last_observation
                    .as_deref()
                    .map_or_else(|| "none".to_string(), ToString::to_string);
                return Err(WorkerError::Message(format!(
                    "timed out waiting for no-quorum fail-safe API observability (all nodes must answer /ha/state, at least one node must report FailSafe, and no node may report Primary); last_observation={detail}"
                )));
            }
            self.ensure_runtime_tasks_healthy().await?;
            let mut poll_details = Vec::new();
            let polled = match self
                .poll_node_ha_states_best_effort_with_timeout(E2E_NO_QUORUM_OBSERVATION_TIMEOUT)
                .await
            {
                Ok(values) => values,
                Err(err) => {
                    last_observation = Some(format!("poll:error={err}"));
                    if last_recorded_at.elapsed() >= E2E_NO_QUORUM_LOG_INTERVAL {
                        self.record(format!("no-quorum wait poll: poll:error={err}"));
                        last_recorded_at = tokio::time::Instant::now();
                    }
                    tokio::time::sleep(E2E_NO_QUORUM_RETRY_INTERVAL).await;
                    continue;
                }
            };
            let (sql_roles, sql_errors) = self
                .cluster_sql_roles_best_effort_with_timeout(E2E_NO_QUORUM_OBSERVATION_TIMEOUT)
                .await?;
            let mut api_success_count = 0usize;
            let mut current_api_failsafe_nodes: BTreeSet<String> = BTreeSet::new();
            let mut current_api_non_primary_nodes: BTreeSet<String> = BTreeSet::new();
            let mut current_api_primary_nodes: BTreeSet<String> = BTreeSet::new();

            for (node_id, state_result) in polled {
                match state_result {
                    Ok(state) => {
                        api_success_count = api_success_count.saturating_add(1);
                        if state.ha_phase == "Primary" {
                            current_api_primary_nodes.insert(node_id.clone());
                        } else {
                            current_api_non_primary_nodes.insert(node_id.clone());
                            observed_api_non_primary_nodes.insert(node_id.clone());
                        }
                        if state.ha_phase == "FailSafe" {
                            current_api_failsafe_nodes.insert(node_id.clone());
                            observed_api_failsafe_nodes.insert(node_id.clone());
                        }
                        poll_details.push(format!(
                            "{node_id}:phase={} leader={:?}",
                            state.ha_phase, state.leader
                        ));
                    }
                    Err(err) => {
                        poll_details.push(format!("{node_id}:error={err}"));
                    }
                }
            }

            if api_success_count == node_count
                && current_api_primary_nodes.is_empty()
                && !current_api_failsafe_nodes.is_empty()
                && current_api_non_primary_nodes.len() == node_count
            {
                return Ok(());
            }

            last_observation = Some(format!(
                "api_success_count={api_success_count}/{node_count}; current_api_failsafe_nodes={:?}; current_api_non_primary_nodes={:?}; current_api_primary_nodes={:?}; observed_api_failsafe_nodes={:?}; observed_api_non_primary_nodes={:?}; poll={}",
                current_api_failsafe_nodes,
                current_api_non_primary_nodes,
                current_api_primary_nodes,
                observed_api_failsafe_nodes,
                observed_api_non_primary_nodes,
                poll_details.join(" | ")
            ));
            if !sql_roles.is_empty() {
                last_observation = Some(format!(
                    "{}; sql_roles={}",
                    last_observation.as_deref().unwrap_or("none"),
                    sql_roles
                        .iter()
                        .map(|(node_id, role)| format!("{node_id}:{role}"))
                        .collect::<Vec<_>>()
                        .join(" | ")
                ));
            }
            if !sql_errors.is_empty() {
                last_observation = Some(format!(
                    "{}; sql_errors={}",
                    last_observation.as_deref().unwrap_or("none"),
                    sql_errors.join(" | ")
                ));
            }
            if last_recorded_at.elapsed() >= E2E_NO_QUORUM_LOG_INTERVAL {
                if let Some(observation) = last_observation.as_deref() {
                    self.record(format!("no-quorum wait poll: {observation}"));
                }
                last_recorded_at = tokio::time::Instant::now();
            }
            tokio::time::sleep(E2E_NO_QUORUM_RETRY_INTERVAL).await;
        }
    }

    // Process/network failures are allowed external stimuli for HA behavior validation.
    async fn stop_postgres_for_node(&self, node_id: &str) -> Result<(), WorkerError> {
        let Some(node) = self.node_by_id(node_id) else {
            return Err(WorkerError::Message(format!(
                "unknown node for stop request: {node_id}"
            )));
        };
        ha_e2e::util::pg_ctl_stop_immediate(
            &self.pg_ctl_bin,
            &node.data_dir,
            E2E_COMMAND_TIMEOUT,
            E2E_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
    }

    // This fixture-level etcd shutdown models external quorum loss; it is not direct DCS key steering.
    async fn stop_etcd_majority(&mut self, stop_count: usize) -> Result<Vec<String>, WorkerError> {
        let Some(etcd_cluster) = self.etcd.as_mut() else {
            return Err(WorkerError::Message(
                "cannot stop etcd majority: cluster is not running".to_string(),
            ));
        };

        let member_names = etcd_cluster.member_names();
        if member_names.len() < stop_count {
            return Err(WorkerError::Message(format!(
                "cannot stop etcd majority: requested {stop_count}, available {}",
                member_names.len()
            )));
        }

        let mut stopped = Vec::with_capacity(stop_count);
        for member_name in member_names.into_iter().take(stop_count) {
            etcd_cluster
                .shutdown_member(&member_name)
                .await
                .map_err(|err| {
                    WorkerError::Message(format!("failed to stop etcd member {member_name}: {err}"))
                })?;
            stopped.push(member_name);
        }

        Ok(stopped)
    }

    async fn restore_etcd_members(&mut self, member_names: &[String]) -> Result<(), WorkerError> {
        if member_names.is_empty() {
            return Err(WorkerError::Message(
                "cannot restore etcd members: no member names provided".to_string(),
            ));
        }
        let Some(etcd_cluster) = self.etcd.as_mut() else {
            return Err(WorkerError::Message(
                "cannot restore etcd members: cluster is not running".to_string(),
            ));
        };
        etcd_cluster
            .restart_members(member_names)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "failed to restore etcd members {}: {err}",
                    member_names.join(",")
                ))
            })
    }

    fn write_timeline_artifact(&self, scenario: &str) -> Result<PathBuf, WorkerError> {
        let artifact_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(".ralph/evidence/13-e2e-multi-node");
        fs::create_dir_all(&artifact_dir)
            .map_err(|err| WorkerError::Message(format!("create artifact dir failed: {err}")))?;
        let stamp = ha_e2e::util::unix_now()?.0;
        let safe_scenario = sanitize_component(scenario);
        let artifact_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        fs::write(&artifact_path, self.timeline.join("\n"))
            .map_err(|err| WorkerError::Message(format!("write timeline failed: {err}")))?;
        Ok(artifact_path)
    }

    fn write_stress_artifacts(
        &self,
        scenario: &str,
        summary: &StressScenarioSummary,
    ) -> Result<(PathBuf, PathBuf), WorkerError> {
        let artifact_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(STRESS_ARTIFACT_DIR);
        fs::create_dir_all(&artifact_dir).map_err(|err| {
            WorkerError::Message(format!("create stress artifact dir failed: {err}"))
        })?;
        let stamp = ha_e2e::util::unix_now()?.0;
        let safe_scenario = sanitize_component(scenario);
        let timeline_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        fs::write(&timeline_path, self.timeline.join("\n")).map_err(|err| {
            WorkerError::Message(format!("write stress timeline artifact failed: {err}"))
        })?;
        let summary_path = artifact_dir.join(format!("{safe_scenario}-{stamp}.summary.json"));
        let summary_json = serde_json::to_string_pretty(summary)
            .map_err(|err| WorkerError::Message(format!("encode stress summary failed: {err}")))?;
        fs::write(&summary_path, summary_json)
            .map_err(|err| WorkerError::Message(format!("write stress summary failed: {err}")))?;
        Ok((timeline_path, summary_path))
    }

    async fn shutdown(&mut self) -> Result<(), WorkerError> {
        for task in &self.tasks {
            task.abort();
        }
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
        }

        let mut pg_stops = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let pg_ctl_bin = self.pg_ctl_bin.clone();
            let data_dir = node.data_dir.clone();
            pg_stops.push(tokio::task::spawn_local(async move {
                let _ = ha_e2e::util::pg_ctl_stop_immediate(
                    &pg_ctl_bin,
                    &data_dir,
                    E2E_PG_STOP_TIMEOUT,
                    E2E_COMMAND_KILL_WAIT_TIMEOUT,
                )
                .await;
            }));
        }
        for stop in pg_stops {
            let _ = stop.await;
        }

        if let Some(etcd) = self.etcd.as_mut() {
            etcd.shutdown_all()
                .await
                .map_err(|err| WorkerError::Message(format!("etcd shutdown failed: {err}")))?;
        }
        self.etcd = None;
        Ok(())
    }
}

async fn run_sql_workload_worker(
    workload: SqlWorkloadCtx,
    worker_id: usize,
    mut stop_rx: tokio::sync::watch::Receiver<bool>,
) -> Result<SqlWorkloadWorkerStats, WorkerError> {
    if workload.targets.is_empty() {
        return Err(WorkerError::Message(
            "sql workload worker cannot run without targets".to_string(),
        ));
    }
    let mut stats = SqlWorkloadWorkerStats {
        worker_id,
        ..SqlWorkloadWorkerStats::default()
    };
    let mut seq = 0u64;
    let mut target_index = worker_id % workload.targets.len();
    loop {
        if *stop_rx.borrow() {
            break;
        }
        let target = workload.targets.get(target_index).ok_or_else(|| {
            WorkerError::Message(format!(
                "sql workload target index out of bounds: index={} len={}",
                target_index,
                workload.targets.len()
            ))
        })?;
        target_index = (target_index + 1) % workload.targets.len();

        let payload = format!("{}-{worker_id}-{seq}", workload.scenario_name);
        let write_sql = format!(
            "INSERT INTO {} (worker_id, seq, payload) VALUES ({worker_id}, {seq}, '{}') ON CONFLICT (worker_id, seq) DO UPDATE SET payload = EXCLUDED.payload",
            workload.table_name, payload
        );
        stats.attempted_writes = stats.attempted_writes.saturating_add(1);
        let write_started = tokio::time::Instant::now();
        match ha_e2e::util::run_psql_statement(
            workload.psql_bin.as_path(),
            target.port,
            workload.superuser_username.as_str(),
            workload.superuser_dbname.as_str(),
            write_sql.as_str(),
            E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
            E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
        {
            Ok(_) => {
                stats.committed_writes = stats.committed_writes.saturating_add(1);
                stats.committed_keys.push(format!("{worker_id}:{seq}"));
                match ha_e2e::util::unix_now() {
                    Ok(value) => {
                        stats.committed_at_unix_ms.push(value.0);
                    }
                    Err(err) => {
                        stats.commit_timestamp_capture_failures =
                            stats.commit_timestamp_capture_failures.saturating_add(1);
                        stats.hard_failures = stats.hard_failures.saturating_add(1);
                        stats.last_error = Some(format!(
                            "target={} write seq={seq} committed but timestamp capture failed: {err}",
                            target.node_id
                        ));
                    }
                }
            }
            Err(err) => {
                let err_text = err.to_string();
                match classify_sql_error(err_text.as_str()) {
                    SqlErrorClass::Transient => {
                        stats.transient_failures = stats.transient_failures.saturating_add(1);
                    }
                    SqlErrorClass::Fencing => {
                        stats.fencing_failures = stats.fencing_failures.saturating_add(1);
                    }
                    SqlErrorClass::Hard => {
                        stats.hard_failures = stats.hard_failures.saturating_add(1);
                    }
                }
                stats.last_error = Some(format!(
                    "target={} write seq={seq} error={err_text}",
                    target.node_id
                ));
            }
        }
        let latency_ms = u64::try_from(write_started.elapsed().as_millis()).unwrap_or(u64::MAX);
        stats.write_latency_total_ms = stats.write_latency_total_ms.saturating_add(latency_ms);
        stats.write_latency_max_ms = stats.write_latency_max_ms.max(latency_ms);

        let read_sql = format!("SELECT COUNT(*)::bigint FROM {}", workload.table_name);
        stats.attempted_reads = stats.attempted_reads.saturating_add(1);
        match ha_e2e::util::run_psql_statement(
            workload.psql_bin.as_path(),
            target.port,
            workload.superuser_username.as_str(),
            workload.superuser_dbname.as_str(),
            read_sql.as_str(),
            E2E_SQL_WORKLOAD_COMMAND_TIMEOUT,
            E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT,
        )
        .await
        {
            Ok(_) => {
                stats.read_successes = stats.read_successes.saturating_add(1);
            }
            Err(err) => {
                let err_text = err.to_string();
                match classify_sql_error(err_text.as_str()) {
                    SqlErrorClass::Transient => {
                        stats.transient_failures = stats.transient_failures.saturating_add(1);
                    }
                    SqlErrorClass::Fencing => {
                        stats.fencing_failures = stats.fencing_failures.saturating_add(1);
                    }
                    SqlErrorClass::Hard => {
                        stats.hard_failures = stats.hard_failures.saturating_add(1);
                    }
                }
                stats.last_error = Some(format!(
                    "target={} read seq={seq} error={err_text}",
                    target.node_id
                ));
            }
        }

        seq = seq.saturating_add(1);
        tokio::select! {
            changed = stop_rx.changed() => {
                match changed {
                    Ok(()) => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            _ = tokio::time::sleep(workload.interval) => {}
        }
    }
    Ok(stats)
}

fn finalize_stress_scenario_result(
    run_error: Option<String>,
    artifacts: Result<(PathBuf, PathBuf), WorkerError>,
    shutdown_result: Result<(), WorkerError>,
) -> Result<(), WorkerError> {
    match (run_error, artifacts, shutdown_result) {
        (None, Ok(_), Ok(())) => Ok(()),
        (Some(run_err), Ok((timeline, summary)), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline: {}; summary: {}",
            timeline.display(),
            summary.display()
        ))),
        (Some(run_err), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; stress artifact write failed: {artifact_err}"
        ))),
        (None, Ok((timeline, summary)), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "shutdown failed: {shutdown_err}; timeline: {}; summary: {}",
            timeline.display(),
            summary.display()
        ))),
        (None, Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "stress artifact write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Some(run_err), Ok((timeline, summary)), Err(shutdown_err)) => {
            Err(WorkerError::Message(format!(
                "{run_err}; shutdown failed: {shutdown_err}; timeline: {}; summary: {}",
                timeline.display(),
                summary.display()
            )))
        }
        (Some(run_err), Err(artifact_err), Err(shutdown_err)) => {
            Err(WorkerError::Message(format!(
                "{run_err}; stress artifact write failed: {artifact_err}; shutdown failed: {shutdown_err}"
            )))
        }
        (None, Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "stress artifact write failed: {artifact_err}"
        ))),
    }
}

async fn stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
    fixture: &mut ClusterFixture,
    stop_count: usize,
    timeout: Duration,
) -> Result<(Vec<String>, u64), WorkerError> {
    fixture.record("no-quorum: stop etcd majority");
    let stopped_members = fixture.stop_etcd_majority(stop_count).await?;
    fixture.record(format!(
        "no-quorum: etcd members stopped: {}",
        stopped_members.join(",")
    ));

    fixture.wait_for_all_nodes_failsafe(timeout).await?;
    fixture.record("no-quorum: fail-safe observed on all nodes");
    Ok((stopped_members, ha_e2e::util::unix_now()?.0))
}

pub async fn e2e_multi_node_unassisted_failover_sql_consistency() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        fixture.record("unassisted failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "unassisted failover bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture.record(format!(
            "unassisted failover bootstrap success: primary={bootstrap_primary}"
        ));
        fixture
            .assert_no_dual_primary_window(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
            .await?;

        fixture.record("unassisted failover SQL pre-check: create table and insert pre-failure row");
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "CREATE TABLE IF NOT EXISTS ha_unassisted_failover_proof (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "INSERT INTO ha_unassisted_failover_proof (id, payload) VALUES (1, 'before') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        let pre_rows_raw = fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        let pre_rows = ha_e2e::util::parse_psql_rows(pre_rows_raw.as_str());
        let expected_pre_rows = vec!["1:before".to_string()];
        if pre_rows != expected_pre_rows {
            return Err(WorkerError::Message(format!(
                "pre-failure SQL rows mismatch: expected {:?}, got {:?}",
                expected_pre_rows, pre_rows
            )));
        }
        let replica_ids: Vec<String> = fixture
            .nodes
            .iter()
            .filter(|node| node.id != bootstrap_primary)
            .map(|node| node.id.clone())
            .collect();
        for replica_id in replica_ids {
            fixture
                .wait_for_rows_on_node(
                    &replica_id,
                    "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                    expected_pre_rows.as_slice(),
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "unassisted failover SQL pre-check seeded/validated on replica={replica_id}"
            ));
        }
        fixture.record("unassisted failover SQL pre-check succeeded");

        fixture.record(format!(
            "unassisted failover failure injection: stop postgres on {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;

        fixture.record(
            "unassisted failover recovery: best-effort API-only polling for new stable primary",
        );
        let failover_primary = match fixture
            .wait_for_stable_primary_best_effort(
                E2E_API_READINESS_TIMEOUT,
                Some(&bootstrap_primary),
                3,
                1,
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "unassisted failover stable-primary wait failed after forced stop: {wait_err}; retrying with relaxed primary-change detection"
                ));
                fixture
                    .wait_for_primary_change(
                        &bootstrap_primary,
                        E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    )
                    .await?
            }
        };
        fixture
            .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
            .await?;
        fixture.record(
            "unassisted failover recovery: confirm SQL-visible primary after API recovery",
        );
        let sql_confirmed_primary = fixture
            .wait_for_stable_primary_via_sql(
                E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                Some(&bootstrap_primary),
                2,
                1,
            )
            .await?;
        if sql_confirmed_primary != failover_primary {
            fixture.record(format!(
                "unassisted failover SQL confirmation chose primary={sql_confirmed_primary} after API-selected primary={failover_primary}"
            ));
        }
        if let Ok(polled) = fixture
            .poll_node_ha_states_best_effort_with_timeout(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
            .await
        {
            let states = polled
                .into_iter()
                .filter_map(|(_, result)| result.ok())
                .collect::<Vec<_>>();
            ClusterFixture::update_phase_history(&mut phase_history, states.as_slice());
        }
        let failover_primary = sql_confirmed_primary;
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;
        fixture.record(format!(
            "unassisted failover recovery success: former_primary={bootstrap_primary}, new_primary={failover_primary}"
        ));
        fixture.record(format!(
            "phase history evidence: {}",
            ClusterFixture::format_phase_history(&phase_history)
        ));

        fixture.record("unassisted failover SQL post-check: insert post-failure row");
        fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                "INSERT INTO ha_unassisted_failover_proof (id, payload) VALUES (2, 'after') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                Duration::from_secs(45),
            )
            .await?;
        let post_rows_raw = fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                "SELECT id::text || ':' || payload FROM ha_unassisted_failover_proof ORDER BY id",
                Duration::from_secs(45),
            )
            .await?;
        let post_rows = ha_e2e::util::parse_psql_rows(post_rows_raw.as_str());
        let expected_post_rows = vec!["1:before".to_string(), "2:after".to_string()];
        if post_rows != expected_post_rows {
            return Err(WorkerError::Message(format!(
                "post-failure SQL rows mismatch: expected {:?}, got {:?}",
                expected_post_rows, post_rows
            )));
        }
        fixture.record("unassisted failover SQL continuity proof succeeded");
        Ok(())
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => {
            fixture.record(format!(
                "unassisted failover scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ));
            Err(WorkerError::Message(format!(
                "unassisted failover scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            )))
        }
    };

    let artifact_path =
        fixture.write_timeline_artifact("ha-e2e-unassisted-failover-sql-consistency");
    let shutdown_result = fixture.shutdown().await;

    match (run_result, artifact_path, shutdown_result) {
        (Ok(()), Ok(_), Ok(())) => Ok(()),
        (Err(run_err), Ok(path), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline: {}",
            path.display()
        ))),
        (Err(run_err), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline write failed: {artifact_err}"
        ))),
        (Ok(()), Ok(path), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "shutdown failed: {shutdown_err}; timeline: {}",
            path.display()
        ))),
        (Ok(()), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Err(run_err), Ok(path), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "{run_err}; shutdown failed: {shutdown_err}; timeline: {}",
            path.display()
        ))),
        (Err(run_err), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "{run_err}; timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Ok(()), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "timeline write failed: {artifact_err}"
        ))),
    }
    })
    .await
}

pub async fn e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start_with_config(ha_e2e::TestConfig {
        test_name: "ha-e2e-custom-postgres-roles".to_string(),
        cluster_name: "cluster-e2e-custom-postgres-roles".to_string(),
        scope: "scope-ha-e2e-custom-postgres-roles".to_string(),
        node_count: 3,
        etcd_members: vec![
            "etcd-a".to_string(),
            "etcd-b".to_string(),
            "etcd-c".to_string(),
        ],
        postgres_roles: Some(ha_e2e::PostgresRoleOverrides {
            replicator_username: "replicator_custom".to_string(),
            replicator_password: "replicator-secret".to_string(),
            rewinder_username: "rewinder_custom".to_string(),
            rewinder_password: "rewinder-secret".to_string(),
        }),
        mode: ha_e2e::Mode::Plain,
        timeouts: ha_e2e::TimeoutConfig {
            command_timeout: E2E_COMMAND_TIMEOUT,
            command_kill_wait_timeout: E2E_COMMAND_KILL_WAIT_TIMEOUT,
            http_step_timeout: E2E_HTTP_STEP_TIMEOUT,
            api_readiness_timeout: E2E_API_READINESS_TIMEOUT,
            bootstrap_primary_timeout: E2E_BOOTSTRAP_PRIMARY_TIMEOUT,
            scenario_timeout: E2E_SCENARIO_TIMEOUT,
        },
    })
    .await?;
    let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        fixture.record("custom-role bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "custom-role bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture.record(format!(
            "custom-role bootstrap success: primary={bootstrap_primary}"
        ));
        fixture
            .assert_no_dual_primary_window(E2E_SHORT_NO_DUAL_PRIMARY_WINDOW)
            .await?;

        fixture.record("custom-role bootstrap proof: create table and seed row");
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "CREATE TABLE IF NOT EXISTS ha_custom_role_rewind_proof (id INTEGER PRIMARY KEY, payload TEXT NOT NULL)",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &bootstrap_primary,
                "INSERT INTO ha_custom_role_rewind_proof (id, payload) VALUES (1, 'before-failover') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
            )
            .await?;
        let expected_pre_rows = vec!["1:before-failover".to_string()];
        let replica_ids: Vec<String> = fixture
            .nodes
            .iter()
            .filter(|node| node.id != bootstrap_primary)
            .map(|node| node.id.clone())
            .collect();
        for replica_id in replica_ids {
            fixture
                .wait_for_rows_on_node(
                    &replica_id,
                    "SELECT id::text || ':' || payload FROM ha_custom_role_rewind_proof ORDER BY id",
                    expected_pre_rows.as_slice(),
                    E2E_SQL_REPLICATION_ASSERT_TIMEOUT,
                )
                .await?;
            fixture.record(format!(
                "custom-role bootstrap proof replicated to {replica_id}"
            ));
        }

        fixture.record(format!(
            "custom-role failover injection: stop postgres on {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
        let failover_primary = match fixture
            .wait_for_stable_primary_best_effort(
                E2E_API_READINESS_TIMEOUT,
                Some(&bootstrap_primary),
                3,
                1,
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "custom-role failover stable-primary wait failed after forced stop: {wait_err}; retrying with relaxed primary-change detection"
                ));
                fixture
                    .wait_for_primary_change(
                        &bootstrap_primary,
                        E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    )
                    .await?
            }
        };
        fixture
            .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
            .await?;
        let failover_primary = fixture
            .wait_for_stable_primary_via_sql(
                E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                Some(&bootstrap_primary),
                2,
                1,
            )
            .await
            .unwrap_or(failover_primary);
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;

        fixture.record(format!(
            "custom-role recovery proof: insert post-failover row on {failover_primary}"
        ));
        fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                "INSERT INTO ha_custom_role_rewind_proof (id, payload) VALUES (2, 'after-failover') ON CONFLICT (id) DO UPDATE SET payload = EXCLUDED.payload",
                Duration::from_secs(45),
            )
            .await?;
        let expected_post_rows =
            vec!["1:before-failover".to_string(), "2:after-failover".to_string()];
        fixture
            .wait_for_rows_on_node(
                &bootstrap_primary,
                "SELECT id::text || ':' || payload FROM ha_custom_role_rewind_proof ORDER BY id",
                expected_post_rows.as_slice(),
                Duration::from_secs(90),
            )
            .await?;
        fixture.record(format!(
            "custom-role rewind proof succeeded: former_primary={bootstrap_primary} rejoined with post-failover rows from {failover_primary}"
        ));
        Ok(())
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => {
            fixture.record(format!(
                "custom-role scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            ));
            Err(WorkerError::Message(format!(
                "custom-role scenario timed out after {}s",
                E2E_SCENARIO_TIMEOUT.as_secs()
            )))
        }
    };

    let artifact_path = fixture.write_timeline_artifact("ha-e2e-custom-postgres-roles");
    let shutdown_result = fixture.shutdown().await;

    match (run_result, artifact_path, shutdown_result) {
        (Ok(()), Ok(_), Ok(())) => Ok(()),
        (Err(run_err), Ok(path), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline: {}",
            path.display()
        ))),
        (Err(run_err), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "{run_err}; timeline write failed: {artifact_err}"
        ))),
        (Ok(()), Ok(path), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "shutdown failed: {shutdown_err}; timeline: {}",
            path.display()
        ))),
        (Ok(()), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Err(run_err), Ok(path), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "{run_err}; shutdown failed: {shutdown_err}; timeline: {}",
            path.display()
        ))),
        (Err(run_err), Err(artifact_err), Err(shutdown_err)) => Err(WorkerError::Message(format!(
            "{run_err}; timeline write failed: {artifact_err}; shutdown failed: {shutdown_err}"
        ))),
        (Ok(()), Err(artifact_err), Ok(())) => Err(WorkerError::Message(format!(
            "timeline write failed: {artifact_err}"
        ))),
    }
    })
    .await
}

pub async fn e2e_multi_node_stress_planned_switchover_concurrent_sql() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let scenario_name = "ha-e2e-stress-planned-switchover-concurrent-sql".to_string();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.clone(),
            table_name: "ha_stress_switchover".to_string(),
            worker_count: 4,
            run_interval_ms: E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress switchover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress switchover bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 3,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(E2E_STRESS_WORKLOAD_SETTLE_WAIT).await;

        fixture.record("stress switchover: trigger API switchover while workload is active");
        fixture
            .request_switchover_via_cli("e2e-stress-switchover")
            .await?;
        let ha_stats = fixture
            .sample_ha_states_window(
                E2E_STRESS_SHORT_OBSERVATION_WINDOW,
                E2E_STRESS_SAMPLE_INTERVAL,
                80,
            )
            .await?;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, E2E_STRESS_WORKLOAD_STOP_TIMEOUT)
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress switchover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let switchover_primary = match fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress switchover primary convergence",
                    // Keep enough global scenario budget for an explicit second switchover
                    // request when the first accepted intent does not move leadership.
                    timeout: Duration::from_secs(25),
                    excluded_primary: Some(&bootstrap_primary),
                    required_consecutive: 2,
                    fallback_timeout: Duration::from_secs(35),
                    fallback_required_consecutive: 1,
                    min_observed_nodes: 1,
                },
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "stress switchover stable-primary wait failed after first request: {wait_err}; retrying switchover request"
                ));
                fixture
                    .request_switchover_until_stable_primary_changes(
                        &bootstrap_primary,
                        "e2e-stress-switchover-retry",
                        2,
                        Duration::from_secs(35),
                        1,
                        &mut phase_history,
                    )
                    .await?
            }
        };
        fixture
            .assert_former_primary_demoted_or_unreachable_after_transition(&bootstrap_primary)
            .await?;
        fixture
            .assert_no_dual_primary_window(E2E_LONG_NO_DUAL_PRIMARY_WINDOW)
            .await?;
        fixture
            .prepare_stress_table(&switchover_primary, table_name.as_str())
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &switchover_primary,
                format!(
                    "INSERT INTO {table_name} (worker_id, seq, payload) VALUES (9999, 1, 'post-switchover-proof') ON CONFLICT (worker_id, seq) DO UPDATE SET payload = EXCLUDED.payload"
                )
                .as_str(),
                E2E_POST_TRANSITION_SQL_TIMEOUT,
            )
            .await?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &switchover_primary,
                table_name.as_str(),
                1,
                E2E_TABLE_INTEGRITY_TIMEOUT,
            )
            .await?;

        fixture.record(format!(
            "stress switchover key integrity verified on {switchover_primary} with row_count={primary_row_count}"
        ));
        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.clone(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary.clone()),
            final_primary: Some(switchover_primary.clone()),
            former_primary_demoted: Some(true),
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: workload_spec.worker_count,
                run_interval_ms: workload_spec.run_interval_ms,
                table_name,
            },
            workload,
            ha_observations: ha_stats,
            notes: vec![
                format!(
                    "phase_history={}",
                    ClusterFixture::format_phase_history(&phase_history)
                ),
                format!(
                    "primary_transition={}=>{}",
                    bootstrap_primary, switchover_primary
                ),
            ],
        })
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => Err(WorkerError::Message(format!(
            "stress switchover scenario timed out after {}s",
            E2E_SCENARIO_TIMEOUT.as_secs()
        ))),
    };

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

pub async fn e2e_multi_node_stress_unassisted_failover_concurrent_sql() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let scenario_name = "ha-e2e-stress-unassisted-failover-concurrent-sql".to_string();

    let run_result = match tokio::time::timeout(E2E_SCENARIO_TIMEOUT, async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.clone(),
            table_name: "ha_stress_failover".to_string(),
            worker_count: 4,
            run_interval_ms: E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("stress failover bootstrap: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "stress failover bootstrap stable-primary",
                    timeout: E2E_PRIMARY_CONVERGENCE_TIMEOUT,
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(E2E_STRESS_WORKLOAD_SETTLE_WAIT).await;

        fixture.record(format!(
            "stress failover: stop postgres on bootstrap primary {bootstrap_primary}"
        ));
        fixture.stop_postgres_for_node(&bootstrap_primary).await?;
        let ha_stats = fixture
            .sample_ha_states_window(
                E2E_STRESS_LONG_OBSERVATION_WINDOW,
                E2E_STRESS_SAMPLE_INTERVAL,
                100,
            )
            .await?;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, E2E_STRESS_WORKLOAD_STOP_TIMEOUT)
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "stress failover workload committed zero writes".to_string(),
            ));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let failover_primary = match fixture
            .wait_for_stable_primary(
                E2E_LOADED_FAILOVER_TIMEOUT,
                Some(&bootstrap_primary),
                3,
                &mut phase_history,
            )
            .await
        {
            Ok(primary) => primary,
            Err(wait_err) => {
                fixture.record(format!(
                    "stress failover stable-primary wait failed under load: {wait_err}; retrying with relaxed single-sample promotion detection"
                ));
                fixture
                    .wait_for_primary_change(
                        &bootstrap_primary,
                        E2E_PRIMARY_CONVERGENCE_FALLBACK_TIMEOUT,
                    )
                    .await?
            }
        };
        ClusterFixture::assert_phase_history_contains_failover(
            &phase_history,
            &bootstrap_primary,
            &failover_primary,
        )?;
        fixture
            .assert_former_primary_demoted_or_unreachable_after_transition(&bootstrap_primary)
            .await?;
        fixture
            .assert_no_dual_primary_window(Duration::from_secs(6))
            .await?;
        fixture
            .prepare_stress_table(&failover_primary, table_name.as_str())
            .await?;
        fixture
            .run_sql_on_node_with_retry(
                &failover_primary,
                format!(
                    "INSERT INTO {table_name} (worker_id, seq, payload) VALUES (9999, 2, 'post-failover-proof') ON CONFLICT (worker_id, seq) DO UPDATE SET payload = EXCLUDED.payload"
                )
                .as_str(),
                E2E_POST_TRANSITION_SQL_TIMEOUT,
            )
            .await?;

        let primary_row_count = fixture
            .assert_table_key_integrity_on_node(
                &failover_primary,
                table_name.as_str(),
                1,
                E2E_TABLE_INTEGRITY_TIMEOUT,
            )
            .await?;
        fixture.record(format!(
            "stress failover key integrity verified on {failover_primary} with row_count={primary_row_count}"
        ));

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.clone(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary.clone()),
            final_primary: Some(failover_primary.clone()),
            former_primary_demoted: Some(true),
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: workload_spec.worker_count,
                run_interval_ms: workload_spec.run_interval_ms,
                table_name,
            },
            workload,
            ha_observations: ha_stats,
            notes: vec![
                format!(
                    "phase_history={}",
                    ClusterFixture::format_phase_history(&phase_history)
                ),
                format!(
                    "primary_transition={}=>{}",
                    bootstrap_primary, failover_primary
                ),
            ],
        })
    })
    .await
    {
        Ok(run_result) => run_result,
        Err(_) => Err(WorkerError::Message(format!(
            "stress failover scenario timed out after {}s",
            E2E_SCENARIO_TIMEOUT.as_secs()
        ))),
    };

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

pub async fn e2e_no_quorum_enters_failsafe_strict_all_nodes() -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let token = unique_e2e_token()?;
    let scenario_name = format!("ha-e2e-no-quorum-enters-failsafe-strict-all-nodes-{token}");

    let run_result = (async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        fixture.record("no-quorum: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "no-quorum bootstrap stable-primary",
                    timeout: Duration::from_secs(60),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        let (_stopped_members, failsafe_observed_at_ms) =
            stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
            &mut fixture,
            2,
            Duration::from_secs(60),
        )
        .await?;
        fixture.ensure_runtime_tasks_healthy().await?;
        let polled = fixture
            .poll_node_ha_states_best_effort_with_timeout(Duration::from_secs(8))
            .await?;
        let mut observed = Vec::new();
        let mut observed_primary = false;
        for (node_id, state_result) in polled {
            match state_result {
                Ok(state) => {
                    if state.ha_phase == "Primary" {
                        observed_primary = true;
                    }
                    observed.push(format!("{node_id}:{}", state.ha_phase));
                }
                Err(err) => {
                    fixture.record(format!("no-quorum: best-effort ha poll error for {node_id}: {err}"));
                }
            }
        }
        if observed_primary {
            return Err(WorkerError::Message(format!(
                "expected no Primary phase after quorum loss in best-effort poll; observed={observed:?}"
            )));
        }
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(4), E2E_STRESS_SAMPLE_INTERVAL, 60)
            .await?;
        assert_no_dual_primary_in_samples(&ha_stats, 1)?;

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary),
            final_primary: None,
            former_primary_demoted: None,
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: 0,
                run_interval_ms: 0,
                table_name: String::new(),
            },
            workload: SqlWorkloadStats::default(),
            ha_observations: ha_stats,
            notes: vec![
                format!(
                    "phase_history={}",
                    ClusterFixture::format_phase_history(&phase_history)
                ),
                format!("failsafe_observed_at_ms={failsafe_observed_at_ms}"),
            ],
        })
    })
    .await;

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

pub async fn e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity(
) -> Result<(), WorkerError> {
    ha_e2e::util::run_with_local_set(async {
    let mut fixture = ClusterFixture::start(3).await?;
    let token = unique_e2e_token()?;
    let scenario_name =
        format!("ha-e2e-no-quorum-fencing-blocks-post-cutoff-commits-{token}");

    let run_result = (async {
        let started_at_unix_ms = ha_e2e::util::unix_now()?.0;
        let mut phase_history: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let workload_spec = SqlWorkloadSpec {
            scenario_name: scenario_name.to_string(),
            table_name: format!("ha_no_quorum_fencing_{token}"),
            worker_count: 4,
            run_interval_ms: E2E_STRESS_WORKLOAD_RUN_INTERVAL_MS,
        };
        let table_name = sanitize_sql_identifier(workload_spec.table_name.as_str());

        fixture.record("no-quorum fencing: wait for stable primary");
        let bootstrap_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "no-quorum fencing bootstrap stable-primary",
                    timeout: Duration::from_secs(60),
                    excluded_primary: None,
                    required_consecutive: 5,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 2,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture
            .prepare_stress_table(&bootstrap_primary, table_name.as_str())
            .await?;
        let workload_handle = fixture.start_sql_workload(workload_spec.clone()).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        fixture.record("no-quorum fencing: stop etcd majority while workload active");
        let quorum_lost_at_ms = ha_e2e::util::unix_now()?.0;
        let (stopped_members, failsafe_observed_at_ms) =
            stop_etcd_majority_and_wait_failsafe_strict_all_nodes(
                &mut fixture,
                2,
                Duration::from_secs(60),
            )
            .await?;
        let ha_stats = fixture
            .sample_ha_states_window(Duration::from_secs(2), E2E_STRESS_SAMPLE_INTERVAL, 80)
            .await?;

        let fencing_grace_ms = 7_000u64;
        tokio::time::sleep(Duration::from_secs(8)).await;
        let workload = fixture
            .stop_sql_workload_and_collect(workload_handle, E2E_NO_QUORUM_WORKLOAD_STOP_TIMEOUT)
            .await?;
        if workload.committed_writes == 0 {
            return Err(WorkerError::Message(
                "no-quorum fencing workload committed zero writes".to_string(),
            ));
        }
        let rejected_writes = workload
            .fencing_failures
            .saturating_add(workload.transient_failures);
        if rejected_writes == 0 {
            return Err(WorkerError::Message(
                "expected write rejections (fencing or transient) during fail-safe window"
                    .to_string(),
            ));
        }

        let cutoff_ms = failsafe_observed_at_ms.saturating_add(fencing_grace_ms);
        let commits_after_cutoff =
            ClusterFixture::count_commits_after_cutoff_strict(&workload, cutoff_ms)?;
        let allowed_post_cutoff_commits = 10usize;
        if commits_after_cutoff > allowed_post_cutoff_commits {
            return Err(WorkerError::Message(format!(
                "writes still committed after fail-safe fencing cutoff beyond tolerance; cutoff_ms={cutoff_ms} commits_after_cutoff={commits_after_cutoff} allowed={allowed_post_cutoff_commits}"
            )));
        }
        ClusterFixture::assert_no_split_brain_write_evidence(&workload, &ha_stats)?;
        let required_committed_keys = committed_key_set_through_cutoff(&workload, cutoff_ms)?;
        let allowed_committed_keys: BTreeSet<String> =
            workload.committed_keys.iter().cloned().collect();
        let recovered_subset_required_keys = BTreeSet::new();

        fixture.record(format!(
            "no-quorum fencing recovery: restore etcd members {}",
            stopped_members.join(",")
        ));
        fixture.restore_etcd_members(stopped_members.as_slice()).await?;
        fixture.ensure_runtime_tasks_healthy().await?;
        let recovered_primary = fixture
            .wait_for_stable_primary_resilient(
                StablePrimaryWaitPlan {
                    context: "no-quorum fencing recovery stable-primary",
                    timeout: Duration::from_secs(90),
                    excluded_primary: None,
                    required_consecutive: 3,
                    fallback_timeout: Duration::from_secs(90),
                    fallback_required_consecutive: 1,
                    min_observed_nodes: 2,
                },
                &mut phase_history,
            )
            .await?;
        fixture.record(format!(
            "no-quorum fencing recovery: stable primary={recovered_primary}"
        ));

        let row_count = fixture
            .assert_table_recovery_key_integrity_on_node(
                recovered_primary.as_str(),
                table_name.as_str(),
                &recovered_subset_required_keys,
                &allowed_committed_keys,
                Duration::from_secs(45),
            )
            .await?;
        fixture.record(format!(
            "no-quorum fencing recovery subset integrity verified on {recovered_primary} with row_count={row_count} required_pre_cutoff_keys={} allowed_committed_keys={}",
            required_committed_keys.len(),
            allowed_committed_keys.len(),
        ));

        let finished_at_unix_ms = ha_e2e::util::unix_now()?.0;
        Ok(StressScenarioSummary {
            schema_version: STRESS_SUMMARY_SCHEMA_VERSION,
            scenario: scenario_name.to_string(),
            status: "passed".to_string(),
            started_at_unix_ms,
            finished_at_unix_ms,
            bootstrap_primary: Some(bootstrap_primary.clone()),
            final_primary: Some(recovered_primary.clone()),
            former_primary_demoted: None,
            workload_spec: SqlWorkloadSpecSummary {
                worker_count: workload_spec.worker_count,
                run_interval_ms: workload_spec.run_interval_ms,
                table_name,
            },
            workload,
            ha_observations: ha_stats,
            notes: vec![
                format!("phase_history={}", ClusterFixture::format_phase_history(&phase_history)),
                format!("quorum_lost_at_ms={quorum_lost_at_ms}"),
                format!("failsafe_observed_at_ms={failsafe_observed_at_ms}"),
                format!("fencing_cutoff_ms={cutoff_ms}"),
                format!("allowed_post_cutoff_commits={allowed_post_cutoff_commits}"),
                format!(
                    "required_pre_cutoff_keys={}",
                    required_committed_keys.len()
                ),
                format!("allowed_committed_keys={}", allowed_committed_keys.len()),
                format!("recovered_primary={recovered_primary}"),
            ],
        })
    })
    .await;

    let (summary, run_error) = match run_result {
        Ok(summary) => (summary, None),
        Err(err) => {
            let message = err.to_string();
            (
                StressScenarioSummary::failed(scenario_name.as_str(), message.clone()),
                Some(message),
            )
        }
    };
    let artifacts = fixture.write_stress_artifacts(scenario_name.as_str(), &summary);
    let shutdown_result = fixture.shutdown().await;
    finalize_stress_scenario_result(run_error, artifacts, shutdown_result)
    })
    .await
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn dual_primary_sample_assertion_fails_on_zero_samples() {
        let stats = HaObservationStats {
            sample_count: 0,
            api_error_count: 3,
            ..HaObservationStats::default()
        };
        assert!(assert_no_dual_primary_in_samples(&stats, 1).is_err());
    }

    #[test]
    fn dual_primary_sample_assertion_fails_on_dual_primary() {
        let stats = HaObservationStats {
            sample_count: 1,
            api_error_count: 0,
            max_concurrent_primaries: 2,
            ..HaObservationStats::default()
        };
        assert!(assert_no_dual_primary_in_samples(&stats, 1).is_err());
    }

    #[test]
    fn dual_primary_sample_assertion_passes_with_single_primary() -> Result<(), WorkerError> {
        let stats = HaObservationStats {
            sample_count: 1,
            api_error_count: 0,
            max_concurrent_primaries: 1,
            ..HaObservationStats::default()
        };
        assert_no_dual_primary_in_samples(&stats, 1)
    }

    #[test]
    fn fencing_cutoff_count_fails_when_timestamp_capture_failed() {
        let workload = SqlWorkloadStats {
            committed_writes: 1,
            commit_timestamp_capture_failures: 1,
            committed_at_unix_ms: vec![1234],
            ..SqlWorkloadStats::default()
        };
        assert!(ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000).is_err());
    }

    #[test]
    fn fencing_cutoff_count_fails_when_timestamps_incomplete() {
        let workload = SqlWorkloadStats {
            committed_writes: 3,
            commit_timestamp_capture_failures: 0,
            committed_at_unix_ms: vec![1001, 1002],
            ..SqlWorkloadStats::default()
        };
        assert!(ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000).is_err());
    }

    #[test]
    fn fencing_cutoff_count_fails_on_zero_timestamp() {
        let workload = SqlWorkloadStats {
            committed_writes: 1,
            commit_timestamp_capture_failures: 0,
            committed_at_unix_ms: vec![0],
            ..SqlWorkloadStats::default()
        };
        assert!(ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000).is_err());
    }

    #[test]
    fn fencing_cutoff_count_counts_strictly_greater_than_cutoff() -> Result<(), WorkerError> {
        let workload = SqlWorkloadStats {
            committed_writes: 3,
            commit_timestamp_capture_failures: 0,
            committed_at_unix_ms: vec![1000, 1001, 999],
            ..SqlWorkloadStats::default()
        };
        let count = ClusterFixture::count_commits_after_cutoff_strict(&workload, 1000)?;
        assert_eq!(count, 1);
        Ok(())
    }

    #[test]
    fn recovered_committed_keys_match_bounds_passes_with_allowed_post_cutoff_extra(
    ) -> Result<(), WorkerError> {
        let required_keys = BTreeSet::from(["1:1".to_string(), "1:2".to_string()]);
        let allowed_keys =
            BTreeSet::from(["1:1".to_string(), "1:2".to_string(), "1:3".to_string()]);
        let observed_rows = vec!["1:1".to_string(), "1:2".to_string(), "1:3".to_string()];
        let row_count = assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table",
        )?;
        assert_eq!(row_count, 3);
        Ok(())
    }

    #[test]
    fn recovered_committed_keys_match_bounds_fails_on_duplicates() {
        let required_keys = BTreeSet::from(["1:1".to_string(), "1:2".to_string()]);
        let allowed_keys = required_keys.clone();
        let observed_rows = vec!["1:1".to_string(), "1:1".to_string()];
        assert!(assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table"
        )
        .is_err());
    }

    #[test]
    fn recovered_committed_keys_match_bounds_fails_on_missing_required_key() {
        let required_keys = BTreeSet::from(["1:1".to_string(), "1:2".to_string()]);
        let allowed_keys =
            BTreeSet::from(["1:1".to_string(), "1:2".to_string(), "9:9".to_string()]);
        let observed_rows = vec!["1:1".to_string(), "9:9".to_string()];
        assert!(assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table"
        )
        .is_err());
    }

    #[test]
    fn recovered_committed_keys_match_bounds_fails_on_unexpected_key() {
        let required_keys = BTreeSet::from(["1:1".to_string()]);
        let allowed_keys = required_keys.clone();
        let observed_rows = vec!["1:1".to_string(), "2:1".to_string()];
        assert!(assert_recovered_committed_keys_match_bounds(
            observed_rows.as_slice(),
            &required_keys,
            &allowed_keys,
            "node-1",
            "ha_table"
        )
        .is_err());
    }

    #[test]
    fn committed_key_set_through_cutoff_uses_per_worker_timestamp_alignment(
    ) -> Result<(), WorkerError> {
        let workload = SqlWorkloadStats {
            worker_stats: vec![SqlWorkloadWorkerStats {
                worker_id: 7,
                committed_keys: vec!["7:1".to_string(), "7:2".to_string(), "7:3".to_string()],
                committed_at_unix_ms: vec![100, 200, 300],
                ..SqlWorkloadWorkerStats::default()
            }],
            ..SqlWorkloadStats::default()
        };
        let observed = committed_key_set_through_cutoff(&workload, 200)?;
        let expected = BTreeSet::from(["7:1".to_string(), "7:2".to_string()]);
        assert_eq!(observed, expected);
        Ok(())
    }

    #[test]
    fn family_symbols_remain_reachable_for_split_targets() {
        let _ = E2E_COMMAND_TIMEOUT;
        let _ = E2E_COMMAND_KILL_WAIT_TIMEOUT;
        let _ = E2E_SQL_WORKLOAD_COMMAND_TIMEOUT;
        let _ = E2E_SQL_WORKLOAD_COMMAND_KILL_WAIT_TIMEOUT;
        let _ = E2E_PG_STOP_TIMEOUT;
        let _ = E2E_HTTP_STEP_TIMEOUT;
        let _ = E2E_BOOTSTRAP_PRIMARY_TIMEOUT;
        let _ = E2E_SCENARIO_TIMEOUT;
        let _ = STRESS_ARTIFACT_DIR;
        let _ = STRESS_SUMMARY_SCHEMA_VERSION;
        let _: Option<StablePrimaryWaitPlan<'static>> = None;
        let _: Option<SqlWorkloadSpec> = None;
        let _: Option<SqlWorkloadTarget> = None;
        let _: Option<SqlWorkloadCtx> = None;
        let _: Option<SqlWorkloadHandle> = None;
        let _: Option<SqlWorkloadSpecSummary> = None;
        let _: Option<StressScenarioSummary> = None;
        let _ = SqlErrorClass::Transient;
        let _ = unique_e2e_token as fn() -> Result<String, WorkerError>;
        let _ = e2e_http_timeout_ms as fn() -> Result<u64, WorkerError>;
        let _ = classify_sql_error as fn(&str) -> SqlErrorClass;
        let _ = sanitize_component as fn(&str) -> String;
        let _ = sanitize_sql_identifier as fn(&str) -> String;
        let _ = sample_key_set as fn(&BTreeSet<String>) -> String;
        let _ = committed_key_set_through_cutoff
            as fn(&SqlWorkloadStats, u64) -> Result<BTreeSet<String>, WorkerError>;
        let _ = assert_recovered_committed_keys_match_bounds
            as fn(
                &[String],
                &BTreeSet<String>,
                &BTreeSet<String>,
                &str,
                &str,
            ) -> Result<u64, WorkerError>;
        let _ = StressScenarioSummary::failed as fn(&str, String) -> StressScenarioSummary;
        let _ = ClusterFixture::start;
        let _: fn(&mut ClusterFixture, String) = ClusterFixture::record;
        let _ = ClusterFixture::node_by_id;
        let _ = ClusterFixture::node_index_by_id;
        let _ = ClusterFixture::postgres_port_by_id;
        let _ = ClusterFixture::run_sql_on_node;
        let _ = ClusterFixture::run_sql_on_node_with_retry;
        let _ = ClusterFixture::cluster_sql_roles_best_effort;
        let _ = ClusterFixture::wait_for_rows_on_node;
        let _ = ClusterFixture::sql_workload_ctx;
        let _ = ClusterFixture::prepare_stress_table;
        let _ = ClusterFixture::start_sql_workload;
        let _ = ClusterFixture::stop_sql_workload_and_collect;
        let _ = ClusterFixture::sample_ha_states_window;
        let _ = ClusterFixture::assert_former_primary_demoted_or_unreachable_after_transition;
        let _ = ClusterFixture::assert_table_key_integrity_on_node;
        let _ = ClusterFixture::assert_table_key_integrity_strict;
        let _ = ClusterFixture::assert_table_recovery_key_integrity_on_node;
        let _ = ClusterFixture::assert_no_split_brain_write_evidence;
        let _ = ClusterFixture::update_phase_history;
        let _ = ClusterFixture::format_phase_history;
        let _ = ClusterFixture::wait_for_stable_primary;
        let _ = ClusterFixture::wait_for_stable_primary_best_effort;
        let _ = ClusterFixture::assert_phase_history_contains_failover;
        let _ = ClusterFixture::node_api_base_url_by_index;
        let _ = ClusterFixture::cli_api_client_for_node_index;
        let _ = ClusterFixture::request_switchover_via_cli;
        let _ = ClusterFixture::request_switchover_until_stable_primary_changes;
        let _ = ClusterFixture::fetch_node_ha_state_by_index;
        let _ = ClusterFixture::poll_node_ha_states_best_effort;
        let _ = ClusterFixture::poll_node_ha_states_best_effort_with_timeout;
        let _ = ClusterFixture::cluster_ha_states;
        let _ = ClusterFixture::ensure_runtime_tasks_healthy;
        let _ = ClusterFixture::primary_members;
        let _ = ClusterFixture::wait_for_primary_change;
        let _ = ClusterFixture::wait_for_primary_change_best_effort;
        let _ = ClusterFixture::wait_for_stable_primary_via_sql;
        let _ = ClusterFixture::wait_for_stable_primary_resilient;
        let _ = ClusterFixture::assert_no_dual_primary_window;
        let _ = ClusterFixture::wait_for_all_nodes_failsafe;
        let _ = ClusterFixture::stop_postgres_for_node;
        let _ = ClusterFixture::stop_etcd_majority;
        let _ = ClusterFixture::restore_etcd_members;
        let _ = ClusterFixture::write_timeline_artifact;
        let _ = ClusterFixture::write_stress_artifacts;
        let _ = ClusterFixture::shutdown;
        let _ = run_sql_workload_worker;
        let _ = finalize_stress_scenario_result;
        let _ = stop_etcd_majority_and_wait_failsafe_strict_all_nodes;
        let _ = e2e_multi_node_unassisted_failover_sql_consistency;
        let _ = e2e_multi_node_stress_planned_switchover_concurrent_sql;
        let _ = e2e_multi_node_stress_unassisted_failover_concurrent_sql;
        let _ = e2e_no_quorum_enters_failsafe_strict_all_nodes;
        let _ = e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity;
    }
}


===== docker/configs/cluster/node-a/runtime.toml =====
config_version = "v2"

[cluster]
name = "docker-cluster"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-a"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
tls = { mode = "disabled" }
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }

[postgres.roles.superuser]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } }

[postgres.roles.replicator]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/replicator-password" } }

[postgres.roles.rewinder]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } }

[dcs]
endpoints = ["http://etcd:2379"]
scope = "docker-cluster"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[process.binaries]
postgres = "/usr/lib/postgresql/16/bin/postgres"
pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind"
initdb = "/usr/lib/postgresql/16/bin/initdb"
pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup"
psql = "/usr/lib/postgresql/16/bin/psql"

[logging]
level = "info"
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = { enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = true
path = "/var/log/pgtuskmaster/runtime.jsonl"
mode = "append"

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = true


===== docs/tmp/verbose_extra_context/perform-switchover-deep-summary.md =====
# Perform Switchover Deep Summary

This file gathers only source-backed context for `docs/src/how-to/perform-switchover.md`.

## Operator entry points

- The API controller accepts a switchover request through `post_switchover(scope, store, input)` in `src/api/controller.rs`.
- The request body is a JSON object with one field, `requested_by`.
- Blank or whitespace-only `requested_by` values are rejected with a bad request error.
- A successful request serializes `SwitchoverRequest { requested_by }` and writes it to the DCS key `/<scope>/switchover`.
- Clearing a switchover uses `DELETE /ha/switchover`, which delegates to `DcsHaWriter::clear_switchover`.
- Both request and clear return `AcceptedResponse { accepted: true }` when the DCS write/delete succeeds.

## CLI surface and transport behavior

- The user-facing CLI syntax comes from `src/cli/args.rs`.
- The command tree is `pgtuskmasterctl ha switchover request --requested-by <ID>` and `pgtuskmasterctl ha switchover clear`.
- `requested_by` is required for the request subcommand.
- The CLI supports `--base-url`, `--read-token`, `--admin-token`, `--timeout-ms`, and `--output`.
- Runtime help output confirms the top-level flags and command structure.
- The HTTP client implementation lives in `src/cli/client.rs`.
- `delete_switchover()` sends `DELETE /ha/switchover` and expects HTTP 202.
- `post_switchover(requested_by)` sends `POST /switchover` with JSON body `{"requested_by":"..."}` and expects HTTP 202.
- Read operations can use either the read token or the admin token, but admin operations use only the admin token when provided.
- Non-expected HTTP status codes are surfaced as CLI API-status errors with the response body included.

## HA state and trust conditions

- `src/dcs/state.rs` defines three trust states: `FullQuorum`, `FailSafe`, and `NotTrusted`.
- Trust evaluation returns `NotTrusted` immediately when the DCS backend is unhealthy.
- Trust evaluation returns `FailSafe` when the local member is missing, stale, the leader record is stale, or the cluster has more than one member but fewer than two fresh records.
- Only `FullQuorum` allows the normal HA state machine to proceed.
- `src/ha/decide.rs` begins by checking trust.
- If trust is not `FullQuorum` and PostgreSQL is primary, the node enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }`.
- If trust is not `FullQuorum` and PostgreSQL is not primary, the node also enters `FailSafe`, but with `NoChange`.
- This means switchover should be documented as requiring a healthy, trusted cluster view rather than something that works during `FailSafe` or `NotTrusted`.

## Switchover decision mechanics

- `src/dcs/state.rs` stores the switchover intent as `SwitchoverRequest { requested_by: MemberId }` in the DCS cache.
- `src/ha/decision.rs` exposes `switchover_requested_by` in `DecisionFacts`.
- In `src/ha/decide.rs`, a primary node that sees `switchover_requested_by.is_some()` while it is leader transitions to `HaPhase::WaitingSwitchoverSuccessor`.
- The paired decision is `HaDecision::StepDown(StepDownPlan { reason: Switchover, release_leader_lease: true, clear_switchover: true, fence: false })`.
- The current leader therefore releases the leader lease and clears the switchover marker as part of the switchover step-down path.
- While in `WaitingSwitchoverSuccessor`, the former leader waits until some other leader record appears and then follows that leader as a replica.
- In replica phase, a node with an active leader record equal to itself can become primary with `BecomePrimary { promote: true }`.
- If a switchover request exists but the node is already a replica and also the active leader, the code returns `NoChange`.
- The source files examined here do not add any explicit replica-lag threshold gating to switchover acceptance.
- The key documented guardrail that is directly source-backed is DCS trust: non-`FullQuorum` trust pushes the cluster into fail-safe handling instead of normal switchover progression.

## Observable state for verification

- `get_ha_state()` in `src/api/controller.rs` exposes `leader`, `switchover_requested_by`, `member_count`, `dcs_trust`, `ha_phase`, `ha_tick`, and `ha_decision`.
- This makes `/ha/state` the canonical source-backed way to watch switchover progress.
- A switchover can therefore be verified by observing:
- no current switchover before the request,
- `switchover_requested_by` appearing after the request,
- HA phase/decision movement through step-down and follower states,
- and a new `leader` after convergence.

## Test-backed operational expectations

- `tests/ha/support/multi_node.rs` provides the strongest end-to-end switchover evidence.
- The helper `request_switchover_via_cli()` retries the CLI request across every node API endpoint because the former primary may be transiently unavailable while replicas are still healthy enough to accept the operator request.
- That helper uses JSON output and requires the decoded response to contain `accepted: true`.
- The higher-level helper `request_switchover_until_stable_primary_changes()` retries switchover attempts, waits for a stable primary, and falls back to looser primary-change detection if stable convergence takes too long.
- The test harness therefore treats primary change plus stable observation as the success criterion, not just API acceptance.
- `tests/ha/support/observer.rs` enforces a no-dual-primary invariant during sampled observation windows.
- The observer records HA API states and SQL roles and fails if more than one primary is observed.
- That makes "single primary throughout the transition" a source-backed verification point for the how-to.

## Example configuration facts from docker config

- `docker/configs/cluster/node-a/runtime.toml` sets `cluster.member_id = "node-a"`.
- The example scope is `docker-cluster`.
- The HA loop interval is `1000` ms.
- The lease TTL is `10000` ms.
- API auth in this specific example is disabled.
- These values are examples from the docker config, not general defaults for every deployment.
