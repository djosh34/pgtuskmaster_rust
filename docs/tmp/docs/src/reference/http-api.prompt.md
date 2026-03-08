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

docs/src/reference/http-api.md

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


===== src/api/worker.rs =====
use std::{sync::Arc, time::Duration};

use rustls::ServerConfig;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::{
    api::{
        controller::{delete_switchover, get_ha_state, post_switchover, SwitchoverRequestInput},
        fallback::{get_fallback_cluster, post_fallback_heartbeat, FallbackHeartbeatInput},
        ApiError,
    },
    config::{ApiAuthConfig, ApiTlsMode, RuntimeConfig},
    dcs::store::DcsStore,
    debug_api::{snapshot::SystemSnapshot, view::build_verbose_payload},
    logging::{AppEvent, AppEventHeader, LogHandle, SeverityText, StructuredFields},
    state::{StateSubscriber, WorkerError},
};

const API_LOOP_POLL_INTERVAL: Duration = Duration::from_millis(10);
const API_ACCEPT_TIMEOUT: Duration = Duration::from_millis(1);
const API_REQUEST_READ_TIMEOUT: Duration = Duration::from_millis(100);
const API_TLS_CLIENT_HELLO_PEEK_TIMEOUT: Duration = Duration::from_millis(10);
const API_REQUEST_ID_MAX_LEN: usize = 128;
const HTTP_REQUEST_MAX_BYTES: usize = 1024 * 1024;
const HTTP_REQUEST_HEADER_LIMIT_BYTES: usize = 16 * 1024;
const HTTP_REQUEST_SCRATCH_BUFFER_BYTES: usize = 4096;
const HTTP_REQUEST_HEADER_CAPACITY: usize = 64;

#[derive(Clone, Debug, Default)]
struct ApiRoleTokens {
    read_token: Option<String>,
    admin_token: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum ApiEventKind {
    StepOnceFailed,
    ConnectionAccepted,
    RequestParseFailed,
    ResponseSent,
    AuthDecision,
    TlsClientCertMissing,
    TlsHandshakeFailed,
}

impl ApiEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StepOnceFailed => "api.step_once_failed",
            Self::ConnectionAccepted => "api.connection_accepted",
            Self::RequestParseFailed => "api.request_parse_failed",
            Self::ResponseSent => "api.response_sent",
            Self::AuthDecision => "api.auth_decision",
            Self::TlsClientCertMissing => "api.tls_client_cert_missing",
            Self::TlsHandshakeFailed => "api.tls_handshake_failed",
        }
    }
}

fn api_event(
    kind: ApiEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "api", result),
    )
}

pub struct ApiWorkerCtx {
    listener: TcpListener,
    poll_interval: Duration,
    scope: String,
    member_id: String,
    config_subscriber: StateSubscriber<RuntimeConfig>,
    dcs_store: Box<dyn DcsStore>,
    debug_snapshot_subscriber: Option<StateSubscriber<SystemSnapshot>>,
    tls_mode_override: Option<ApiTlsMode>,
    tls_acceptor: Option<TlsAcceptor>,
    role_tokens: Option<ApiRoleTokens>,
    require_client_cert: bool,
    log: LogHandle,
}

impl ApiWorkerCtx {
    pub fn contract_stub(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
    ) -> Self {
        Self::new(
            listener,
            config_subscriber,
            dcs_store,
            LogHandle::disabled(),
        )
    }

    pub(crate) fn new(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
        log: LogHandle,
    ) -> Self {
        let scope = config_subscriber.latest().value.dcs.scope.clone();
        let member_id = config_subscriber.latest().value.cluster.member_id.clone();
        Self {
            listener,
            poll_interval: API_LOOP_POLL_INTERVAL,
            scope,
            member_id,
            config_subscriber,
            dcs_store,
            debug_snapshot_subscriber: None,
            tls_mode_override: None,
            tls_acceptor: None,
            role_tokens: None,
            require_client_cert: false,
            log,
        }
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr, WorkerError> {
        self.listener
            .local_addr()
            .map_err(|err| WorkerError::Message(format!("api local_addr failed: {err}")))
    }

    pub fn configure_tls(
        &mut self,
        mode: ApiTlsMode,
        server_config: Option<Arc<ServerConfig>>,
    ) -> Result<(), WorkerError> {
        match mode {
            ApiTlsMode::Disabled => {
                self.tls_mode_override = Some(ApiTlsMode::Disabled);
                self.tls_acceptor = None;
                Ok(())
            }
            ApiTlsMode::Optional | ApiTlsMode::Required => {
                let cfg = server_config.ok_or_else(|| {
                    WorkerError::Message(
                        "tls mode optional/required requires a server tls config".to_string(),
                    )
                })?;
                self.tls_mode_override = Some(mode);
                self.tls_acceptor = Some(TlsAcceptor::from(cfg));
                Ok(())
            }
        }
    }

    pub fn configure_role_tokens(
        &mut self,
        read_token: Option<String>,
        admin_token: Option<String>,
    ) -> Result<(), WorkerError> {
        let read = normalize_optional_token(read_token)?;
        let admin = normalize_optional_token(admin_token)?;

        if read.is_none() && admin.is_none() {
            self.role_tokens = None;
            return Ok(());
        }

        self.role_tokens = Some(ApiRoleTokens {
            read_token: read,
            admin_token: admin,
        });
        Ok(())
    }

    pub fn set_require_client_cert(&mut self, required: bool) {
        self.require_client_cert = required;
    }

    pub(crate) fn set_ha_snapshot_subscriber(
        &mut self,
        subscriber: StateSubscriber<SystemSnapshot>,
    ) {
        self.debug_snapshot_subscriber = Some(subscriber);
    }
}

pub async fn run(mut ctx: ApiWorkerCtx) -> Result<(), WorkerError> {
    loop {
        if let Err(err) = step_once(&mut ctx).await {
            let fatal = is_fatal_api_step_error(&err);
            let mut event = api_event(
                ApiEventKind::StepOnceFailed,
                "failed",
                if fatal {
                    SeverityText::Error
                } else {
                    SeverityText::Warn
                },
                "api step failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(api_base_fields(&ctx).into_attributes());
            fields.insert("error", err.to_string());
            fields.insert("fatal", fatal);
            ctx.log
                .emit_app_event("api_worker::run", event)
                .map_err(|emit_err| {
                    WorkerError::Message(format!("api step failure log emit failed: {emit_err}"))
                })?;

            if fatal {
                return Err(err);
            }
        }
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub async fn step_once(ctx: &mut ApiWorkerCtx) -> Result<(), WorkerError> {
    let (stream, peer) = match tokio::time::timeout(API_ACCEPT_TIMEOUT, ctx.listener.accept()).await
    {
        Ok(Ok((stream, peer))) => (stream, peer),
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!("api accept failed: {err}")));
        }
        Err(_elapsed) => return Ok(()),
    };

    let cfg = ctx.config_subscriber.latest().value;
    let mut accept_event = api_event(
        ApiEventKind::ConnectionAccepted,
        "ok",
        SeverityText::Debug,
        "api connection accepted",
    );
    let fields = accept_event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert(
        "api.tls_mode",
        format!("{:?}", effective_tls_mode(ctx, &cfg)).to_lowercase(),
    );
    ctx.log
        .emit_app_event("api_worker::step_once", accept_event)
        .map_err(|err| WorkerError::Message(format!("api accept log emit failed: {err}")))?;

    let mut stream = match accept_connection(ctx, &cfg, peer, stream).await? {
        Some(stream) => stream,
        None => return Ok(()),
    };

    let request =
        match tokio::time::timeout(API_REQUEST_READ_TIMEOUT, stream.read_http_request()).await {
            Ok(Ok(req)) => req,
            Ok(Err(message)) => {
                let mut event = api_event(
                    ApiEventKind::RequestParseFailed,
                    "failed",
                    SeverityText::Warn,
                    "api request parse failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(api_base_fields(ctx).into_attributes());
                fields.insert("api.peer_addr", peer.to_string());
                fields.insert("error", message.clone());
                ctx.log
                    .emit_app_event("api_worker::step_once", event)
                    .map_err(|err| {
                        WorkerError::Message(format!("api parse failure log emit failed: {err}"))
                    })?;
                let response = HttpResponse::text(400, "Bad Request", message);
                stream.write_http_response(response).await?;
                return Ok(());
            }
            Err(_elapsed) => return Ok(()),
        };

    match authorize_request(ctx, &cfg, &request) {
        AuthDecision::Allowed => {}
        AuthDecision::Unauthorized => {
            emit_api_auth_decision(ctx, peer, &request, "unauthorized")?;
            let response = HttpResponse::text(401, "Unauthorized", "unauthorized");
            stream.write_http_response(response).await?;
            return Ok(());
        }
        AuthDecision::Forbidden => {
            emit_api_auth_decision(ctx, peer, &request, "forbidden")?;
            let response = HttpResponse::text(403, "Forbidden", "forbidden");
            stream.write_http_response(response).await?;
            return Ok(());
        }
    }

    emit_api_auth_decision(ctx, peer, &request, "allowed")?;

    let response = route_request(ctx, &cfg, peer, request);
    let status_code = response.status;
    stream.write_http_response(response).await?;

    let mut event = api_event(
        ApiEventKind::ResponseSent,
        "ok",
        SeverityText::Debug,
        "api response sent",
    );
    let fields = event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert("api.status_code", u64::from(status_code));
    ctx.log
        .emit_app_event("api_worker::step_once", event)
        .map_err(|err| WorkerError::Message(format!("api response log emit failed: {err}")))?;
    Ok(())
}

fn api_base_fields(ctx: &ApiWorkerCtx) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.member_id.clone());
    fields
}

fn extract_request_id(request: &HttpRequest) -> Option<String> {
    request
        .headers
        .iter()
        .find(|(name, _value)| name.eq_ignore_ascii_case("x-request-id"))
        .map(|(_name, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.len() > API_REQUEST_ID_MAX_LEN {
                value[..API_REQUEST_ID_MAX_LEN].to_string()
            } else {
                value
            }
        })
}

fn auth_header_present(request: &HttpRequest) -> bool {
    request
        .headers
        .iter()
        .any(|(name, _value)| name.eq_ignore_ascii_case("authorization"))
}

fn route_template(request: &HttpRequest) -> String {
    let (path, _query) = split_path_and_query(&request.path);
    format!("{} {}", request.method, path)
}

fn emit_api_auth_decision(
    ctx: &ApiWorkerCtx,
    peer: std::net::SocketAddr,
    request: &HttpRequest,
    decision: &str,
) -> Result<(), WorkerError> {
    let mut event = api_event(
        ApiEventKind::AuthDecision,
        "ok",
        SeverityText::Debug,
        "api auth decision",
    );
    let fields = event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert("api.method", request.method.clone());
    fields.insert("api.route_template", route_template(request));
    fields.insert("api.auth.header_present", auth_header_present(request));
    fields.insert("api.auth.result", decision.to_string());
    fields.insert(
        "api.auth.required_role",
        format!("{:?}", endpoint_role(request)).to_lowercase(),
    );
    if let Some(request_id) = extract_request_id(request) {
        fields.insert("api.request_id", request_id);
    }
    ctx.log
        .emit_app_event("api_worker::authorize_request", event)
        .map_err(|err| WorkerError::Message(format!("api auth log emit failed: {err}")))?;
    Ok(())
}

fn is_fatal_api_step_error(err: &WorkerError) -> bool {
    let message = err.to_string();
    message.contains("api accept failed")
        || message.contains("tls mode requires a configured tls acceptor")
        || message.contains("api local_addr failed")
}

fn route_request(
    ctx: &mut ApiWorkerCtx,
    cfg: &RuntimeConfig,
    _peer: std::net::SocketAddr,
    request: HttpRequest,
) -> HttpResponse {
    let (path, query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover") => {
            let input = match serde_json::from_slice::<SwitchoverRequestInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            match post_switchover(&ctx.scope, &mut *ctx.dcs_store, input) {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("DELETE", "/ha/switchover") => match delete_switchover(&ctx.scope, &mut *ctx.dcs_store) {
            Ok(value) => HttpResponse::json(202, "Accepted", &value),
            Err(err) => api_error_to_http(err),
        },
        ("GET", "/ha/state") => {
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let snapshot = subscriber.latest();
            let response = get_ha_state(&snapshot);
            HttpResponse::json(200, "OK", &response)
        }
        ("GET", "/fallback/cluster") => {
            let view = get_fallback_cluster(cfg);
            HttpResponse::json(200, "OK", &view)
        }
        ("POST", "/fallback/heartbeat") => {
            let input = match serde_json::from_slice::<FallbackHeartbeatInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            match post_fallback_heartbeat(input) {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("GET", "/debug/snapshot") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let snapshot = subscriber.latest();
            HttpResponse::text(200, "OK", format!("{:#?}", snapshot))
        }
        ("GET", "/debug/verbose") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let since_sequence = match parse_since_sequence(query) {
                Ok(value) => value,
                Err(message) => return HttpResponse::text(400, "Bad Request", message),
            };
            let snapshot = subscriber.latest();
            let payload = build_verbose_payload(&snapshot, since_sequence);
            HttpResponse::json(200, "OK", &payload)
        }
        ("GET", "/debug/ui") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            HttpResponse::html(200, "OK", debug_ui_html())
        }
        _ => HttpResponse::text(404, "Not Found", "not found"),
    }
}

fn api_error_to_http(err: ApiError) -> HttpResponse {
    match err {
        ApiError::BadRequest(message) => HttpResponse::text(400, "Bad Request", message),
        ApiError::DcsStore(message) => HttpResponse::text(503, "Service Unavailable", message),
        ApiError::Internal(message) => HttpResponse::text(500, "Internal Server Error", message),
    }
}

fn split_path_and_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((head, tail)) => (head, Some(tail)),
        None => (path, None),
    }
}

fn parse_since_sequence(query: Option<&str>) -> Result<Option<u64>, String> {
    let Some(query) = query else {
        return Ok(None);
    };

    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == "since" {
            let parsed = value
                .parse::<u64>()
                .map_err(|err| format!("invalid since query parameter: {err}"))?;
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}

fn debug_ui_html() -> &'static str {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>PGTuskMaster Debug UI</title>
  <style>
    :root {
      --bg: radial-gradient(circle at 10% 10%, #162132, #081019 55%, #06090f 100%);
      --panel: rgba(16, 26, 40, 0.92);
      --line: rgba(139, 190, 255, 0.22);
      --text: #d8e6ff;
      --muted: #89a3c4;
      --ok: #4bd18b;
      --warn: #f0bc5e;
      --err: #ff7070;
      --accent: #5ec3ff;
      --font: "JetBrains Mono", "Fira Mono", Menlo, monospace;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: var(--font);
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      padding: 14px;
    }
    .layout {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 12px;
      max-width: 1300px;
      margin: 0 auto;
    }
    .panel {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 12px;
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.04);
    }
    .panel h2 {
      margin: 0 0 10px 0;
      font-size: 14px;
      letter-spacing: 0.04em;
      color: var(--accent);
      text-transform: uppercase;
    }
    .metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
    .metric {
      border: 1px solid var(--line);
      border-radius: 9px;
      padding: 8px;
      background: rgba(0,0,0,0.2);
    }
    .metric .label { font-size: 11px; color: var(--muted); text-transform: uppercase; }
    .metric .value { margin-top: 6px; font-size: 16px; font-weight: 700; }
    .badge {
      display: inline-flex;
      align-items: center;
      padding: 2px 8px;
      border-radius: 999px;
      font-size: 11px;
      border: 1px solid var(--line);
      margin-left: 8px;
    }
    .badge.ok { color: var(--ok); border-color: color-mix(in oklab, var(--ok), black 40%); }
    .badge.warn { color: var(--warn); border-color: color-mix(in oklab, var(--warn), black 40%); }
    .badge.err { color: var(--err); border-color: color-mix(in oklab, var(--err), black 40%); }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 12px;
    }
    th, td {
      text-align: left;
      padding: 6px;
      border-bottom: 1px solid rgba(255,255,255,0.08);
      vertical-align: top;
      word-break: break-word;
    }
    th { color: var(--muted); }
    .timeline { max-height: 260px; overflow: auto; }
    .full { grid-column: 1 / -1; }
    @media (max-width: 760px) {
      body { padding: 8px; }
      .metrics { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <div class="layout">
    <section class="panel full" id="meta-panel">
      <h2>Runtime Meta <span id="meta-badge" class="badge warn">loading</span></h2>
      <div class="metrics">
        <div class="metric"><div class="label">Lifecycle</div><div class="value" id="m-lifecycle">-</div></div>
        <div class="metric"><div class="label">Sequence</div><div class="value" id="m-seq">-</div></div>
        <div class="metric"><div class="label">Generated (ms)</div><div class="value" id="m-ts">-</div></div>
      </div>
    </section>
    <section class="panel" id="config-panel"><h2>Config</h2><div id="config-body">-</div></section>
    <section class="panel" id="pginfo-panel"><h2>PgInfo</h2><div id="pginfo-body">-</div></section>
    <section class="panel" id="dcs-panel"><h2>DCS</h2><div id="dcs-body">-</div></section>
    <section class="panel" id="process-panel"><h2>Process</h2><div id="process-body">-</div></section>
    <section class="panel" id="ha-panel"><h2>HA</h2><div id="ha-body">-</div></section>
    <section class="panel full timeline" id="timeline-panel">
      <h2>Timeline</h2>
      <table>
        <thead><tr><th>Seq</th><th>At</th><th>Category</th><th>Message</th></tr></thead>
        <tbody id="timeline-body"></tbody>
      </table>
    </section>
    <section class="panel full timeline" id="changes-panel">
      <h2>Changes</h2>
      <table>
        <thead><tr><th>Seq</th><th>At</th><th>Domain</th><th>Versions</th><th>Summary</th></tr></thead>
        <tbody id="changes-body"></tbody>
      </table>
    </section>
  </div>
  <script>
    const state = { since: 0 };
    const byId = (id) => document.getElementById(id);
    const asText = (value) => (value === null || value === undefined ? "-" : String(value));
    const badge = (label, cls) => {
      const el = byId("meta-badge");
      el.textContent = label;
      el.className = `badge ${cls}`;
    };
    function renderKeyValue(id, entries) {
      byId(id).innerHTML = entries
        .map(([k, v]) => `<div><strong>${k}</strong>: ${asText(v)}</div>`)
        .join("");
    }
    function renderRows(id, rows, mapRow) {
      byId(id).innerHTML = rows.map(mapRow).join("");
    }
    function render(payload) {
      byId("m-lifecycle").textContent = asText(payload.meta.app_lifecycle);
      byId("m-seq").textContent = asText(payload.meta.sequence);
      byId("m-ts").textContent = asText(payload.meta.generated_at_ms);
      badge("connected", "ok");

      renderKeyValue("config-body", [
        ["member", payload.config.member_id],
        ["cluster", payload.config.cluster_name],
        ["scope", payload.config.scope],
        ["version", payload.config.version],
        ["debug", payload.config.debug_enabled],
        ["tls", payload.config.tls_enabled]
      ]);
      renderKeyValue("pginfo-body", [
        ["variant", payload.pginfo.variant],
        ["worker", payload.pginfo.worker],
        ["sql", payload.pginfo.sql],
        ["readiness", payload.pginfo.readiness],
        ["summary", payload.pginfo.summary]
      ]);
      renderKeyValue("dcs-body", [
        ["worker", payload.dcs.worker],
        ["trust", payload.dcs.trust],
        ["members", payload.dcs.member_count],
        ["leader", payload.dcs.leader],
        ["switchover", payload.dcs.has_switchover_request]
      ]);
      renderKeyValue("process-body", [
        ["worker", payload.process.worker],
        ["state", payload.process.state],
        ["running_job", payload.process.running_job_id],
        ["last_outcome", payload.process.last_outcome]
      ]);
      renderKeyValue("ha-body", [
        ["worker", payload.ha.worker],
        ["phase", payload.ha.phase],
        ["tick", payload.ha.tick],
        ["decision", payload.ha.decision],
        ["decision_detail", payload.ha.decision_detail ?? "<none>"],
        ["planned_actions", payload.ha.planned_actions]
      ]);

      renderRows("timeline-body", payload.timeline, (row) =>
        `<tr><td>${row.sequence}</td><td>${row.at_ms}</td><td>${row.category}</td><td>${row.message}</td></tr>`
      );
      renderRows("changes-body", payload.changes, (row) =>
        `<tr><td>${row.sequence}</td><td>${row.at_ms}</td><td>${row.domain}</td><td>${asText(row.previous_version)} -> ${asText(row.current_version)}</td><td>${row.summary}</td></tr>`
      );

      if (typeof payload.meta.sequence === "number") {
        state.since = Math.max(state.since, payload.meta.sequence);
      }
    }
    async function tick() {
      try {
        const response = await fetch(`/debug/verbose?since=${state.since}`, { cache: "no-store" });
        if (!response.ok) {
          badge(`http-${response.status}`, "warn");
          return;
        }
        const payload = await response.json();
        render(payload);
      } catch (err) {
        badge("offline", "err");
        console.error("debug ui fetch failed", err);
      }
    }
    tick();
    setInterval(tick, 900);
  </script>
</body>
</html>"#
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndpointRole {
    Read,
    Admin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthDecision {
    Allowed,
    Unauthorized,
    Forbidden,
}

fn authorize_request(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
    request: &HttpRequest,
) -> AuthDecision {
    let tokens = resolve_role_tokens(ctx, cfg);
    if tokens.read_token.is_none() && tokens.admin_token.is_none() {
        return AuthDecision::Allowed;
    }

    let Some(token) = extract_bearer_token(request) else {
        return AuthDecision::Unauthorized;
    };

    if let Some(expected_admin) = tokens.admin_token.as_deref() {
        if token == expected_admin {
            return AuthDecision::Allowed;
        }
    }

    match endpoint_role(request) {
        EndpointRole::Read => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return AuthDecision::Allowed;
                }
            }
            AuthDecision::Unauthorized
        }
        EndpointRole::Admin => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return AuthDecision::Forbidden;
                }
            }
            AuthDecision::Unauthorized
        }
    }
}

fn resolve_role_tokens(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiRoleTokens {
    if let Some(configured) = ctx.role_tokens.as_ref() {
        return configured.clone();
    }

    match &cfg.api.security.auth {
        ApiAuthConfig::Disabled => ApiRoleTokens {
            read_token: None,
            admin_token: None,
        },
        ApiAuthConfig::RoleTokens(tokens) => ApiRoleTokens {
            read_token: normalize_runtime_token(tokens.read_token.clone()),
            admin_token: normalize_runtime_token(tokens.admin_token.clone()),
        },
    }
}

fn endpoint_role(request: &HttpRequest) -> EndpointRole {
    let (path, _query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover")
        | ("POST", "/fallback/heartbeat")
        | ("DELETE", "/ha/switchover") => EndpointRole::Admin,
        _ => EndpointRole::Read,
    }
}

fn normalize_optional_token(raw: Option<String>) -> Result<Option<String>, WorkerError> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(WorkerError::Message(
                    "role token must not be empty when configured".to_string(),
                ))
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

fn normalize_runtime_token(raw: Option<String>) -> Option<String> {
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

enum ApiConnection {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

impl ApiConnection {
    async fn write_http_response(&mut self, response: HttpResponse) -> Result<(), WorkerError> {
        match self {
            Self::Plain(stream) => write_http_response(stream, response).await,
            Self::Tls(stream) => write_http_response(stream, response).await,
        }
    }

    async fn read_http_request(&mut self) -> Result<HttpRequest, String> {
        match self {
            Self::Plain(stream) => read_http_request(stream).await,
            Self::Tls(stream) => read_http_request(stream).await,
        }
    }
}

async fn accept_connection(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
    peer: std::net::SocketAddr,
    stream: TcpStream,
) -> Result<Option<ApiConnection>, WorkerError> {
    match effective_tls_mode(ctx, cfg) {
        ApiTlsMode::Disabled => Ok(Some(ApiConnection::Plain(stream))),
        ApiTlsMode::Required => {
            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        let mut event = api_event(
                            ApiEventKind::TlsClientCertMissing,
                            "failed",
                            SeverityText::Warn,
                            "tls client cert missing",
                        );
                        let fields = event.fields_mut();
                        fields.append_json_map(api_base_fields(ctx).into_attributes());
                        fields.insert("api.peer_addr", peer.to_string());
                        fields.insert("api.tls_mode", "required");
                        ctx.log
                            .emit_app_event("api_worker::accept_connection", event)
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "api tls missing cert log emit failed: {err}"
                                ))
                            })?;
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(err) => {
                    let mut event = api_event(
                        ApiEventKind::TlsHandshakeFailed,
                        "failed",
                        SeverityText::Warn,
                        "tls handshake failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(api_base_fields(ctx).into_attributes());
                    fields.insert("api.peer_addr", peer.to_string());
                    fields.insert("api.tls_mode", "required");
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("api_worker::accept_connection", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "api tls handshake log emit failed: {emit_err}"
                            ))
                        })?;
                    Ok(None)
                }
            }
        }
        ApiTlsMode::Optional => {
            if !looks_like_tls_client_hello(&stream).await? {
                return Ok(Some(ApiConnection::Plain(stream)));
            }

            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        let mut event = api_event(
                            ApiEventKind::TlsClientCertMissing,
                            "failed",
                            SeverityText::Warn,
                            "tls client cert missing",
                        );
                        let fields = event.fields_mut();
                        fields.append_json_map(api_base_fields(ctx).into_attributes());
                        fields.insert("api.peer_addr", peer.to_string());
                        fields.insert("api.tls_mode", "optional");
                        ctx.log
                            .emit_app_event("api_worker::accept_connection", event)
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "api tls missing cert log emit failed: {err}"
                                ))
                            })?;
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(err) => {
                    let mut event = api_event(
                        ApiEventKind::TlsHandshakeFailed,
                        "failed",
                        SeverityText::Warn,
                        "tls handshake failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(api_base_fields(ctx).into_attributes());
                    fields.insert("api.peer_addr", peer.to_string());
                    fields.insert("api.tls_mode", "optional");
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("api_worker::accept_connection", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "api tls handshake log emit failed: {emit_err}"
                            ))
                        })?;
                    Ok(None)
                }
            }
        }
    }
}

fn effective_tls_mode(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiTlsMode {
    if let Some(mode) = ctx.tls_mode_override {
        return mode;
    }

    cfg.api.security.tls.mode
}

fn require_tls_acceptor(ctx: &ApiWorkerCtx) -> Result<TlsAcceptor, WorkerError> {
    ctx.tls_acceptor.clone().ok_or_else(|| {
        WorkerError::Message("tls mode requires a configured tls acceptor".to_string())
    })
}

fn has_peer_client_cert(stream: &TlsStream<TcpStream>) -> bool {
    let (_, connection) = stream.get_ref();
    connection
        .peer_certificates()
        .map(|certs| !certs.is_empty())
        .unwrap_or(false)
}

async fn looks_like_tls_client_hello(stream: &TcpStream) -> Result<bool, WorkerError> {
    let mut first = [0_u8; 1];
    match tokio::time::timeout(API_TLS_CLIENT_HELLO_PEEK_TIMEOUT, stream.peek(&mut first)).await {
        Err(_) => Ok(false),
        Ok(Ok(0)) => Ok(false),
        Ok(Ok(_)) => Ok(first[0] == 0x16),
        Ok(Err(err)) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(false),
        Ok(Err(err)) => Err(WorkerError::Message(format!("api tls peek failed: {err}"))),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpResponse {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            content_type: "text/plain; charset=utf-8",
            body: body.into().into_bytes(),
        }
    }

    fn json<T: serde::Serialize>(status: u16, reason: &'static str, value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(body) => Self {
                status,
                reason,
                content_type: "application/json",
                body,
            },
            Err(err) => Self::text(
                500,
                "Internal Server Error",
                format!("json encode failed: {err}"),
            ),
        }
    }

    fn html(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            content_type: "text/html; charset=utf-8",
            body: body.into().into_bytes(),
        }
    }
}

async fn write_http_response<S>(stream: &mut S, response: HttpResponse) -> Result<(), WorkerError>
where
    S: AsyncWrite + Unpin,
{
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        response.reason,
        response.content_type,
        response.body.len()
    );
    stream
        .write_all(header.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("api write header failed: {err}")))?;
    stream
        .write_all(&response.body)
        .await
        .map_err(|err| WorkerError::Message(format!("api write body failed: {err}")))?;
    Ok(())
}

async fn read_http_request<S>(stream: &mut S) -> Result<HttpRequest, String>
where
    S: AsyncRead + Unpin,
{
    let mut buffer = Vec::<u8>::new();
    let mut temp = [0u8; HTTP_REQUEST_SCRATCH_BUFFER_BYTES];
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;

    loop {
        if buffer.len() > HTTP_REQUEST_MAX_BYTES {
            return Err("request too large".to_string());
        }

        let n = stream
            .read(&mut temp)
            .await
            .map_err(|err| err.to_string())?;
        if n == 0 {
            return Err("client closed connection".to_string());
        }
        buffer.extend_from_slice(&temp[..n]);

        if header_end.is_none() {
            if let Some(pos) = find_header_end(&buffer) {
                header_end = Some(pos);
            } else if buffer.len() > HTTP_REQUEST_HEADER_LIMIT_BYTES {
                return Err("headers too large".to_string());
            }
        }

        if let Some(end) = header_end {
            if content_length.is_none() {
                content_length = parse_content_length(&buffer).map_err(|err| err.to_string())?;
            }
            let body_len = content_length.unwrap_or(0);
            let required = end.saturating_add(body_len);
            if buffer.len() >= required {
                break;
            }
        }
    }

    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; HTTP_REQUEST_HEADER_CAPACITY];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(&buffer).map_err(|err| err.to_string())?;
    let header_bytes = match status {
        httparse::Status::Complete(bytes) => bytes,
        httparse::Status::Partial => return Err("incomplete http request".to_string()),
    };

    let method = req
        .method
        .ok_or_else(|| "missing http method".to_string())?
        .to_string();
    let path = req
        .path
        .ok_or_else(|| "missing http path".to_string())?
        .to_string();

    let mut parsed_headers = Vec::new();
    for header in req.headers.iter() {
        parsed_headers.push((
            header.name.to_string(),
            String::from_utf8_lossy(header.value).to_string(),
        ));
    }

    let body_len = content_length.unwrap_or(0);
    let body_end = header_bytes
        .checked_add(body_len)
        .ok_or_else(|| "content-length overflow".to_string())?;
    if body_end > buffer.len() {
        return Err("incomplete http body".to_string());
    }

    Ok(HttpRequest {
        method,
        path,
        headers: parsed_headers,
        body: buffer[header_bytes..body_end].to_vec(),
    })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn parse_content_length(buffer: &[u8]) -> Result<Option<usize>, String> {
    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; 64];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(buffer).map_err(|err| err.to_string())?;
    match status {
        httparse::Status::Complete(_bytes) => {}
        httparse::Status::Partial => return Ok(None),
    }

    for header in req.headers.iter() {
        if header.name.eq_ignore_ascii_case("Content-Length") {
            let raw = String::from_utf8_lossy(header.value);
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(Some(0));
            }
            let parsed = trimmed
                .parse::<usize>()
                .map_err(|err| format!("invalid content-length: {err}"))?;
            return Ok(Some(parsed));
        }
    }
    Ok(Some(0))
}

fn extract_bearer_token(request: &HttpRequest) -> Option<String> {
    let header = request
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Authorization"))
        .map(|(_, value)| value.as_str())?;

    let trimmed = header.trim();
    let prefix = "Bearer ";
    if !trimmed.starts_with(prefix) {
        return None;
    }
    Some(trimmed[prefix.len()..].trim().to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use rustls::{pki_types::ServerName, ClientConfig};
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    use crate::logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink};

    use crate::{
        api::worker::{
            step_once, ApiWorkerCtx, HTTP_REQUEST_HEADER_LIMIT_BYTES,
            HTTP_REQUEST_SCRATCH_BUFFER_BYTES,
        },
        config::{ApiAuthConfig, ApiRoleTokensConfig, ApiTlsMode, InlineOrPath, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        dcs::store::{DcsStore, DcsStoreError, WatchEvent},
        debug_api::snapshot::{
            AppLifecycle, DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot,
        },
        ha::{
            decision::HaDecision,
            state::{HaPhase, HaState},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError},
        test_harness::{
            auth::ApiRoleTokens,
            namespace::NamespaceGuard,
            tls::{
                build_adversarial_tls_fixture, build_client_config, build_server_config,
                build_server_config_with_client_auth, write_tls_material,
            },
        },
    };

    #[derive(Clone, Default)]
    struct RecordingStore {
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingStore {
        fn write_count(&self) -> Result<usize, WorkerError> {
            let guard = self
                .writes
                .lock()
                .map_err(|_| WorkerError::Message("writes lock poisoned".to_string()))?;
            Ok(guard.len())
        }

        fn delete_count(&self) -> Result<usize, WorkerError> {
            let guard = self
                .deletes
                .lock()
                .map_err(|_| WorkerError::Message("deletes lock poisoned".to_string()))?;
            Ok(guard.len())
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

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
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

        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_api_listen_addr("127.0.0.1:0")
            .with_api_auth(auth)
            .build()
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: crate::state::WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: Some(5432),
                    hot_standby: Some(false),
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        }
    }

    fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
        DcsState {
            worker: crate::state::WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config: cfg,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: crate::state::WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn sample_ha_state() -> HaState {
        HaState {
            worker: crate::state::WorkerStatus::Running,
            phase: HaPhase::Replica,
            tick: 7,
            decision: HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
        }
    }

    fn sample_debug_snapshot(auth_token: Option<String>) -> SystemSnapshot {
        let cfg = sample_runtime_config(auth_token);
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        SystemSnapshot {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
            generated_at: UnixMillis(1),
            sequence: 2,
            changes: vec![DebugChangeEvent {
                sequence: 1,
                at: UnixMillis(1),
                domain: DebugDomain::Config,
                previous_version: None,
                current_version: Some(cfg_subscriber.latest().version),
                summary: "config initialized".to_string(),
            }],
            timeline: vec![DebugTimelineEntry {
                sequence: 2,
                at: UnixMillis(1),
                domain: DebugDomain::Ha,
                message: "ha reached replica".to_string(),
            }],
        }
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    async fn build_ctx_with_config(
        cfg: RuntimeConfig,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

        let store = RecordingStore::default();
        let ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store.clone()));
        Ok((ctx, store))
    }

    async fn build_ctx_with_config_and_log(
        cfg: RuntimeConfig,
    ) -> Result<(ApiWorkerCtx, RecordingStore, Arc<TestSink>), WorkerError> {
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

        let store = RecordingStore::default();
        let (log, sink) = test_log_handle();
        let ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(store.clone()), log);
        Ok((ctx, store, sink))
    }

    async fn build_ctx(
        auth_token: Option<String>,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        build_ctx_with_config(sample_runtime_config(auth_token)).await
    }

    const MAX_BODY_BYTES: usize = 256 * 1024;
    const MAX_RESPONSE_BYTES: usize = HTTP_REQUEST_HEADER_LIMIT_BYTES + MAX_BODY_BYTES;
    const IO_TIMEOUT: Duration = Duration::from_secs(2);

    #[derive(Debug)]
    struct TestHttpResponse {
        status_code: u16,
        body: Vec<u8>,
    }

    #[derive(Debug)]
    struct ParsedHttpHead {
        status_code: u16,
        content_length: usize,
        body_start: usize,
    }

    fn parse_http_response_head(
        raw: &[u8],
        header_end: usize,
    ) -> Result<ParsedHttpHead, WorkerError> {
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
        let status_code = status_str.parse::<u16>().map_err(|err| {
            WorkerError::Message(format!("response status code parse failed: {err}"))
        })?;
        if !(100..=599).contains(&status_code) {
            return Err(WorkerError::Message(format!(
                "response status code out of range: {status_code}"
            )));
        }

        let header_text = head.get(status_line_end + 2..).ok_or_else(|| {
            WorkerError::Message("response header offset out of bounds".to_string())
        })?;
        let header_text = std::str::from_utf8(header_text)
            .map_err(|err| WorkerError::Message(format!("response headers not utf8: {err}")))?;

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
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                if content_length.is_some() {
                    return Err(WorkerError::Message(
                        "response contains multiple Content-Length headers".to_string(),
                    ));
                }
                let parsed = value.trim().parse::<usize>().map_err(|err| {
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
            let mut scratch = [0u8; HTTP_REQUEST_SCRATCH_BUFFER_BYTES];

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
                    if raw.len() > HTTP_REQUEST_HEADER_LIMIT_BYTES {
                        return Err(WorkerError::Message(format!(
                            "response headers exceeded limit of {HTTP_REQUEST_HEADER_LIMIT_BYTES} bytes"
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
                        let expected =
                            head.body_start.checked_add(head.content_length).ok_or_else(|| {
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

    async fn send_plain_request(
        ctx: &mut ApiWorkerCtx,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<TestHttpResponse, WorkerError> {
        let addr = ctx.local_addr()?;
        let mut client = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        client
            .write_all(request_head.as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("client write header failed: {err}")))?;

        if let Some(body) = body {
            client
                .write_all(&body)
                .await
                .map_err(|err| WorkerError::Message(format!("client write body failed: {err}")))?;
        }

        step_once(ctx).await?;
        read_http_response_framed(&mut client, IO_TIMEOUT).await
    }

    async fn send_tls_request(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<TestHttpResponse, WorkerError> {
        let addr = ctx.local_addr()?;
        let tcp = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        let connector = TlsConnector::from(client_config);
        let server_name = ServerName::try_from(server_name.to_string()).map_err(|err| {
            WorkerError::Message(format!("invalid server name {server_name}: {err}"))
        })?;

        let client = async move {
            let mut tls = connector
                .connect(server_name, tcp)
                .await
                .map_err(|err| WorkerError::Message(format!("tls connect failed: {err}")))?;
            tls.write_all(request_head.as_bytes())
                .await
                .map_err(|err| WorkerError::Message(format!("tls write header failed: {err}")))?;
            if let Some(body) = body {
                tls.write_all(&body)
                    .await
                    .map_err(|err| WorkerError::Message(format!("tls write body failed: {err}")))?;
            }
            read_http_response_framed(&mut tls, IO_TIMEOUT).await
        };

        let (step_result, client_result) = tokio::join!(step_once(ctx), client);
        step_result?;
        client_result
    }

    async fn expect_tls_handshake_failure(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
    ) -> Result<(), WorkerError> {
        let addr = ctx.local_addr()?;
        let tcp = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        let connector = TlsConnector::from(client_config);
        let server_name = ServerName::try_from(server_name.to_string()).map_err(|err| {
            WorkerError::Message(format!("invalid server name {server_name}: {err}"))
        })?;

        let client = async move { connector.connect(server_name, tcp).await };
        let (step_result, client_result) = tokio::join!(step_once(ctx), client);
        step_result?;
        if client_result.is_ok() {
            return Err(WorkerError::Message(
                "expected tls handshake failure, but handshake succeeded".to_string(),
            ));
        }
        Ok(())
    }

    async fn expect_tls_request_rejected(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
    ) -> Result<(), WorkerError> {
        let result = send_tls_request(
            ctx,
            client_config,
            server_name,
            format_get("/fallback/cluster", None),
            None,
        )
        .await;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    Err(WorkerError::Message(format!(
                        "expected tls request rejection, got status {}",
                        response.status_code
                    )))
                } else {
                    Ok(())
                }
            }
            Err(_) => Ok(()),
        }
    }

    fn format_get(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        }
    }

    fn format_post(path: &str, auth: Option<&str>, body: &[u8]) -> String {
        match auth {
            Some(auth_header) => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
            None => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
        }
    }

    fn format_delete(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "DELETE {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("DELETE {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_allow_read_deny_admin() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-role-read-deny")?;

        let (mut ctx, store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let response = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.read_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert_eq!(response.status_code, 403);
        assert_eq!(store.write_count()?, 0);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn auth_decision_logs_do_not_leak_bearer_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-auth-redaction")?;

        let (mut ctx, _store, sink) =
            build_ctx_with_config_and_log(sample_runtime_config(None)).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let secret = "super-secret-token-value";
        let auth_header = format!("Bearer {secret}");
        let response = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(auth_header.as_str())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 401);

        let records = sink
            .snapshot()
            .map_err(|err| WorkerError::Message(format!("log snapshot failed: {err}")))?;

        let auth_decision_present = records.iter().any(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "api.auth_decision")
                .unwrap_or(false)
        });
        if !auth_decision_present {
            return Err(WorkerError::Message(
                "expected api.auth_decision log event, but it was not emitted".to_string(),
            ));
        }

        for record in records {
            let encoded = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("encode log record failed: {err}")))?;
            if encoded.contains(secret) {
                return Err(WorkerError::Message(
                    "bearer token leaked into structured logs".to_string(),
                ));
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_allow_admin() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-role-admin-allow")?;

        let (mut ctx, store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.admin_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert_eq!(response.status_code, 202);
        assert_eq!(store.write_count()?, 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_typed_json_even_when_debug_disabled() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-state-json")?;
        let mut cfg = sample_runtime_config(None);
        cfg.debug.enabled = false;
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 200);
        let decoded: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|err| WorkerError::Message(format!("decode ha state json failed: {err}")))?;
        assert_eq!(decoded["cluster_name"], "cluster-a");
        assert_eq!(decoded["scope"], "scope-a");
        assert_eq!(decoded["self_member_id"], "node-a");
        assert_eq!(decoded["leader"], serde_json::Value::Null);
        assert_eq!(decoded["switchover_requested_by"], serde_json::Value::Null);
        assert_eq!(decoded["member_count"], 0);
        assert_eq!(decoded["dcs_trust"], "full_quorum");
        assert_eq!(decoded["ha_phase"], "replica");
        assert_eq!(decoded["ha_tick"], 7);
        assert_eq!(decoded["ha_decision"]["kind"], "enter_fail_safe");
        assert_eq!(decoded["ha_decision"]["release_leader_lease"], false);
        assert_eq!(decoded["snapshot_sequence"], 2);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-state-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 503);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_leader_routes_are_not_found_and_do_not_mutate_dcs_keys() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-leader-routes-removed")?;
        let (mut ctx, store) = build_ctx(None).await?;

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post("/ha/leader", None, body.as_slice()),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);

        let response =
            send_plain_request(&mut ctx, format_delete("/ha/leader", None), None).await?;
        assert_eq!(response.status_code, 404);

        let response =
            send_plain_request(&mut ctx, format_delete("/ha/switchover", None), None).await?;
        assert_eq!(response.status_code, 202);

        assert_eq!(store.write_count()?, 0);

        assert_eq!(store.delete_count()?, 1);
        let deletes = store.deletes()?;
        assert_eq!(deletes, vec!["/scope-a/switchover"]);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_handle_removed_ha_leader_routes() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-authz-removed-leader-routes")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.read_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.admin_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_legacy_auth_token_fallback_protects_ha_routes() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-legacy-fallback")?;
        let (mut ctx, _store) = build_ctx(Some("legacy-token".to_string())).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_api_tokens_override_legacy_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-api-precedence")?;
        let mut cfg = sample_runtime_config(Some("legacy-token".to_string()));
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some("read-token".to_string()),
            admin_token: Some("admin-token".to_string()),
        });
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer read-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_returns_structured_json_and_since_filter(
    ) -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-verbose-json")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose?since=1", None), None).await?;
        assert_eq!(response.status_code, 200);

        let decoded: serde_json::Value = serde_json::from_slice(&response.body).map_err(|err| {
            WorkerError::Message(format!("decode debug verbose json failed: {err}"))
        })?;
        assert_eq!(decoded["meta"]["schema_version"], "v1");
        assert_eq!(decoded["meta"]["sequence"], 2);
        assert!(decoded["timeline"].is_array());
        assert!(decoded["changes"].is_array());
        assert_eq!(
            decoded["changes"].as_array().map(|value| value.len()),
            Some(0)
        );
        let endpoints = decoded["api"]["endpoints"].as_array().ok_or_else(|| {
            WorkerError::Message("debug verbose payload missing api.endpoints".to_string())
        })?;
        let contains_restore_route = endpoints.iter().any(|value| {
            value
                .as_str()
                .map(|route| route.contains("restore"))
                .unwrap_or(false)
        });
        assert!(!contains_restore_route);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_snapshot_route_is_kept_for_backward_compatibility() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-snapshot-compat")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/snapshot", None), None).await?;
        assert_eq!(response.status_code, 200);
        let body_text = String::from_utf8(response.body)
            .map_err(|err| WorkerError::Message(format!("snapshot body not utf8: {err}")))?;
        assert!(body_text.contains("SystemSnapshot"));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_404_when_debug_disabled() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-disabled-404")?;
        let mut cfg = sample_runtime_config(None);
        cfg.debug.enabled = false;
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 404);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 503);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_ui_route_returns_html_scaffold() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-ui-html")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response = send_plain_request(&mut ctx, format_get("/debug/ui", None), None).await?;
        assert_eq!(response.status_code, 200);
        let html = String::from_utf8(response.body)
            .map_err(|err| WorkerError::Message(format!("ui body not utf8: {err}")))?;
        assert!(html.contains("id=\"meta-panel\""));
        assert!(html.contains("/debug/verbose"));
        assert!(html.contains("id=\"timeline-panel\""));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_routes_require_auth_when_tokens_set() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-authz")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/ui", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.admin_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_disabled_accepts_plain_rejects_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-disabled")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "disabled",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;

        let response =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert_eq!(response.status_code, 200);

        let trusted_client = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx, trusted_client, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_optional_accepts_plain_and_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-optional")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "optional",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Optional,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;

        let response =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert_eq!(response.status_code, 200);

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_rejects_plain() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "required",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let addr = ctx.local_addr()?;
        let mut plain = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
        plain
            .write_all(format_get("/fallback/cluster", None).as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("plain write failed: {err}")))?;
        step_once(&mut ctx).await?;
        let plain_result = read_http_response_framed(&mut plain, IO_TIMEOUT).await;
        if let Ok(plain_response) = plain_result {
            assert_ne!(plain_response.status_code, 200);
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_with_production_tls_builder(
    ) -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required-prod-builder")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "required-prod-builder",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let tls_cfg = crate::config::TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: None,
        };

        let server_cfg = crate::tls::build_rustls_server_config(&tls_cfg).map_err(|err| {
            WorkerError::Message(format!(
                "build production rustls server config failed: {err}"
            ))
        })?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(ApiTlsMode::Required, server_cfg)?;

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_required_works_with_production_tls_builder() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-required-prod-builder")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server-prod-builder",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client-prod-builder",
            Some(fixture.trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.trusted_client.cert_pem.as_bytes()),
            Some(fixture.trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client-prod-builder",
            Some(fixture.untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.key_pem.as_bytes()),
        )?;

        let tls_cfg = crate::config::TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: Some(crate::config::TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                },
                require_client_cert: true,
            }),
        };

        let server_cfg = crate::tls::build_rustls_server_config(&tls_cfg).map_err(|err| {
            WorkerError::Message(format!(
                "build production rustls server config failed: {err}"
            ))
        })?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(ApiTlsMode::Required, server_cfg)?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        let response = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let missing_client_cert_cfg =
            build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.untrusted_client),
            Some(&fixture.untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_wrong_ca_and_hostname_and_expiry_failures() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-failures")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_valid = write_tls_material(
            namespace,
            "valid-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_expired = write_tls_material(
            namespace,
            "expired-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.expired_server.cert_pem.as_bytes()),
            Some(fixture.expired_server.key_pem.as_bytes()),
        )?;

        let (mut ctx_wrong_ca, _store) = build_ctx(None).await?;
        ctx_wrong_ca.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_wrong_ca = build_client_config(&fixture.wrong_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_wrong_ca, client_wrong_ca, "localhost").await?;

        let (mut ctx_hostname, _store) = build_ctx(None).await?;
        ctx_hostname.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_hostname = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_hostname, client_hostname, "not-localhost").await?;

        let (mut ctx_expired, _store) = build_ctx(None).await?;
        ctx_expired.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.expired_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_expired = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_expired, client_expired, "localhost").await?;

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_node_auth_allows_trusted_client_only() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-node-auth")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client",
            Some(fixture.trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.trusted_client.cert_pem.as_bytes()),
            Some(fixture.trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client",
            Some(fixture.untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config_with_client_auth(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
                &fixture.trusted_client_ca.cert,
            )?),
        )?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        let response = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let missing_client_cert_cfg =
            build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.untrusted_client),
            Some(&fixture.untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;

        Ok(())
    }
}


===== src/debug_api/view.rs =====
use serde::Serialize;

use crate::{
    config::RuntimeConfig,
    dcs::state::{DcsState, DcsTrust},
    debug_api::snapshot::{DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot},
    ha::{lower::lower_decision, state::HaState},
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::state::{JobOutcome, ProcessState},
    state::{Versioned, WorkerStatus},
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugVerbosePayload {
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

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugMeta {
    pub(crate) schema_version: &'static str,
    pub(crate) generated_at_ms: u64,
    pub(crate) channel_updated_at_ms: u64,
    pub(crate) channel_version: u64,
    pub(crate) app_lifecycle: String,
    pub(crate) sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfigSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) cluster_name: String,
    pub(crate) member_id: String,
    pub(crate) scope: String,
    pub(crate) debug_enabled: bool,
    pub(crate) tls_enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PgInfoSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) variant: &'static str,
    pub(crate) worker: String,
    pub(crate) sql: String,
    pub(crate) readiness: String,
    pub(crate) timeline: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DcsSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) trust: String,
    pub(crate) member_count: usize,
    pub(crate) leader: Option<String>,
    pub(crate) has_switchover_request: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProcessSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) state: &'static str,
    pub(crate) running_job_id: Option<String>,
    pub(crate) last_outcome: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct HaSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) phase: String,
    pub(crate) tick: u64,
    pub(crate) decision: String,
    pub(crate) decision_detail: Option<String>,
    pub(crate) planned_actions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ApiSection {
    pub(crate) endpoints: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugSection {
    pub(crate) history_changes: usize,
    pub(crate) history_timeline: usize,
    pub(crate) last_sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugChangeView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) domain: String,
    pub(crate) previous_version: Option<u64>,
    pub(crate) current_version: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugTimelineView {
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
            schema_version: "v1",
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
                "/debug/snapshot",
                "/debug/verbose",
                "/debug/ui",
                "/fallback/cluster",
                "/switchover",
                "/ha/state",
                "/ha/switchover",
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
            variant: "Unknown",
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
            variant: "Primary",
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
            variant: "Replica",
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
            state: "Idle",
            running_job_id: None,
            last_outcome: last_outcome.as_ref().map(job_outcome_label),
        },
        ProcessState::Running { worker, active } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Running",
            running_job_id: Some(active.id.0.clone()),
            last_outcome: None,
        },
    }
}

fn to_ha_section(ha: &Versioned<HaState>) -> HaSection {
    let decision = &ha.value.decision;

    HaSection {
        version: ha.version.0,
        updated_at_ms: ha.updated_at.0,
        worker: worker_status_label(&ha.value.worker),
        phase: format!("{:?}", ha.value.phase),
        tick: ha.value.tick,
        decision: decision.label().to_string(),
        decision_detail: decision.detail(),
        planned_actions: lower_decision(decision).len(),
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

fn job_outcome_label(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { id, .. } => format!("Success({})", id.0),
        JobOutcome::Failure { id, error, .. } => format!("Failure({}: {:?})", id.0, error),
        JobOutcome::Timeout { id, .. } => format!("Timeout({})", id.0),
    }
}


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


===== src/debug_api/worker.rs =====
use std::{collections::VecDeque, time::Duration};

use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    debug_api::snapshot::{
        build_snapshot, AppLifecycle, DebugChangeEvent, DebugDomain, DebugSnapshotCtx,
        DebugTimelineEntry, SystemSnapshot,
    },
    ha::{lower::lower_decision, state::HaState},
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{StatePublisher, StateSubscriber, UnixMillis, Version, WorkerError},
};

const DEFAULT_HISTORY_LIMIT: usize = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DebugObservedState {
    app: AppLifecycle,
    config_version: Version,
    config_sig: String,
    pg_version: Version,
    pg_sig: String,
    dcs_version: Version,
    dcs_sig: String,
    process_version: Version,
    process_sig: String,
    ha_version: Version,
    ha_sig: String,
}

pub(crate) struct DebugApiCtx {
    pub(crate) app: AppLifecycle,
    pub(crate) publisher: StatePublisher<SystemSnapshot>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) ha_subscriber: StateSubscriber<HaState>,
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) history_limit: usize,
    sequence: u64,
    last_observed: Option<DebugObservedState>,
    changes: VecDeque<DebugChangeEvent>,
    timeline: VecDeque<DebugTimelineEntry>,
}

pub(crate) struct DebugApiContractStubInputs {
    pub(crate) publisher: StatePublisher<SystemSnapshot>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) ha_subscriber: StateSubscriber<HaState>,
}

impl DebugApiCtx {
    pub(crate) fn contract_stub(inputs: DebugApiContractStubInputs) -> Self {
        let DebugApiContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        } = inputs;

        Self {
            app: AppLifecycle::Starting,
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
            poll_interval: Duration::from_millis(10),
            now: Box::new(|| Ok(UnixMillis(0))),
            history_limit: DEFAULT_HISTORY_LIMIT,
            sequence: 0,
            last_observed: None,
            changes: VecDeque::new(),
            timeline: VecDeque::new(),
        }
    }

    fn next_sequence(&mut self) -> Result<u64, WorkerError> {
        let next = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| WorkerError::Message("debug_api sequence overflow".to_string()))?;
        self.sequence = next;
        Ok(next)
    }

    fn trim_history(&mut self) {
        while self.changes.len() > self.history_limit {
            let _ = self.changes.pop_front();
        }
        while self.timeline.len() > self.history_limit {
            let _ = self.timeline.pop_front();
        }
    }

    fn record_change(
        &mut self,
        now: UnixMillis,
        domain: DebugDomain,
        previous_version: Option<Version>,
        current_version: Option<Version>,
        summary: String,
    ) -> Result<(), WorkerError> {
        let sequence = self.next_sequence()?;
        self.changes.push_back(DebugChangeEvent {
            sequence,
            at: now,
            domain: domain.clone(),
            previous_version,
            current_version,
            summary: summary.clone(),
        });
        self.timeline.push_back(DebugTimelineEntry {
            sequence,
            at: now,
            domain,
            message: summary,
        });
        self.trim_history();
        Ok(())
    }
}

pub(crate) async fn run(mut ctx: DebugApiCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut DebugApiCtx) -> Result<(), WorkerError> {
    let now = (ctx.now)()?;
    let snapshot_ctx = DebugSnapshotCtx {
        app: ctx.app.clone(),
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
        ha: ctx.ha_subscriber.latest(),
    };

    let config_summary = summarize_config(&snapshot_ctx.config.value);
    let pg_summary = summarize_pg(&snapshot_ctx.pg.value);
    let dcs_summary = summarize_dcs(&snapshot_ctx.dcs.value);
    let process_summary = summarize_process(&snapshot_ctx.process.value);
    let ha_summary = summarize_ha(&snapshot_ctx.ha.value);
    let ha_sig = ha_signature(&snapshot_ctx.ha.value);

    let observed = DebugObservedState {
        app: snapshot_ctx.app.clone(),
        config_version: snapshot_ctx.config.version,
        config_sig: config_summary.clone(),
        pg_version: snapshot_ctx.pg.version,
        pg_sig: pg_summary.clone(),
        dcs_version: snapshot_ctx.dcs.version,
        dcs_sig: dcs_summary.clone(),
        process_version: snapshot_ctx.process.version,
        process_sig: process_summary.clone(),
        ha_version: snapshot_ctx.ha.version,
        ha_sig,
    };

    if let Some(previous) = ctx.last_observed.clone() {
        if previous.app != observed.app {
            ctx.record_change(
                now,
                DebugDomain::App,
                None,
                None,
                summarize_app(&observed.app),
            )?;
        }
        if previous.config_sig != observed.config_sig {
            ctx.record_change(
                now,
                DebugDomain::Config,
                Some(previous.config_version),
                Some(observed.config_version),
                config_summary.clone(),
            )?;
        }
        if previous.pg_sig != observed.pg_sig {
            ctx.record_change(
                now,
                DebugDomain::PgInfo,
                Some(previous.pg_version),
                Some(observed.pg_version),
                pg_summary.clone(),
            )?;
        }
        if previous.dcs_sig != observed.dcs_sig {
            ctx.record_change(
                now,
                DebugDomain::Dcs,
                Some(previous.dcs_version),
                Some(observed.dcs_version),
                dcs_summary.clone(),
            )?;
        }
        if previous.process_sig != observed.process_sig {
            ctx.record_change(
                now,
                DebugDomain::Process,
                Some(previous.process_version),
                Some(observed.process_version),
                process_summary.clone(),
            )?;
        }
        if previous.ha_sig != observed.ha_sig {
            ctx.record_change(
                now,
                DebugDomain::Ha,
                Some(previous.ha_version),
                Some(observed.ha_version),
                ha_summary.clone(),
            )?;
        }
    } else {
        ctx.record_change(
            now,
            DebugDomain::App,
            None,
            None,
            summarize_app(&observed.app),
        )?;
        ctx.record_change(
            now,
            DebugDomain::Config,
            None,
            Some(observed.config_version),
            config_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::PgInfo,
            None,
            Some(observed.pg_version),
            pg_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Dcs,
            None,
            Some(observed.dcs_version),
            dcs_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Process,
            None,
            Some(observed.process_version),
            process_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Ha,
            None,
            Some(observed.ha_version),
            ha_summary,
        )?;
    }

    ctx.last_observed = Some(observed);

    let changes = ctx.changes.iter().cloned().collect::<Vec<_>>();
    let timeline = ctx.timeline.iter().cloned().collect::<Vec<_>>();
    let snapshot = build_snapshot(&snapshot_ctx, now, ctx.sequence, &changes, &timeline);

    ctx.publisher
        .publish(snapshot, now)
        .map_err(|err| WorkerError::Message(format!("debug_api publish failed: {err}")))?;
    Ok(())
}

fn summarize_app(app: &AppLifecycle) -> String {
    format!("app={app:?}")
}

fn summarize_config(config: &RuntimeConfig) -> String {
    format!(
        "cluster={} member={} scope={} debug_enabled={} tls_enabled={}",
        config.cluster.name,
        config.cluster.member_id,
        config.dcs.scope,
        config.debug.enabled,
        config.api.security.tls.mode != crate::config::ApiTlsMode::Disabled
    )
}

fn summarize_pg(state: &PgInfoState) -> String {
    match state {
        PgInfoState::Unknown { common } => {
            format!(
                "pg=unknown worker={:?} sql={:?} readiness={:?}",
                common.worker, common.sql, common.readiness
            )
        }
        PgInfoState::Primary {
            common,
            wal_lsn,
            slots,
        } => {
            format!(
                "pg=primary worker={:?} wal_lsn={} slots={}",
                common.worker,
                wal_lsn.0,
                slots.len()
            )
        }
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => {
            format!(
                "pg=replica worker={:?} replay_lsn={} follow_lsn={} upstream={}",
                common.worker,
                replay_lsn.0,
                follow_lsn
                    .map(|value| value.0)
                    .map_or_else(|| "none".to_string(), |value| value.to_string()),
                upstream
                    .as_ref()
                    .map(|value| value.member_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            )
        }
    }
}

fn summarize_dcs(state: &DcsState) -> String {
    format!(
        "dcs worker={:?} trust={:?} members={} leader={} switchover={}",
        state.worker,
        state.trust,
        state.cache.members.len(),
        state
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone())
            .unwrap_or_else(|| "none".to_string()),
        state.cache.switchover.is_some()
    )
}

fn summarize_process(state: &ProcessState) -> String {
    match state {
        ProcessState::Idle {
            worker,
            last_outcome,
        } => {
            format!("process=idle worker={worker:?} last_outcome={last_outcome:?}")
        }
        ProcessState::Running { worker, active } => {
            format!(
                "process=running worker={worker:?} job_id={} kind={:?}",
                active.id.0, active.kind
            )
        }
    }
}

fn summarize_ha(state: &HaState) -> String {
    let decision_detail = state
        .decision
        .detail()
        .unwrap_or_else(|| "<none>".to_string());
    format!(
        "ha worker={:?} phase={:?} tick={} decision={} detail={} planned_actions={}",
        state.worker,
        state.phase,
        state.tick,
        state.decision.label(),
        decision_detail,
        lower_decision(&state.decision).len()
    )
}

fn ha_signature(state: &HaState) -> String {
    let decision_detail = state
        .decision
        .detail()
        .unwrap_or_else(|| "<none>".to_string());
    format!(
        "ha worker={:?} phase={:?} decision={} detail={}",
        state.worker,
        state.phase,
        state.decision.label(),
        decision_detail
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::{ApiTlsMode, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        debug_api::snapshot::{AppLifecycle, DebugDomain, SystemSnapshot},
        ha::decision::HaDecision,
        ha::state::{HaPhase, HaState},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError, WorkerStatus},
    };

    use super::{DebugApiContractStubInputs, DebugApiCtx};

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

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_snapshot() -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));

        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(|| Ok(UnixMillis(2)));
        ctx.app = AppLifecycle::Running;

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert_eq!(latest.updated_at, UnixMillis(2));
        assert_eq!(latest.value.app, AppLifecycle::Running);
        assert_eq!(latest.value.sequence, 6);
        assert_eq!(latest.value.changes.len(), 6);
        assert_eq!(latest.value.timeline.len(), 6);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_keeps_history_when_versions_unchanged(
    ) -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(3)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let first = subscriber.latest();
        super::step_once(&mut ctx).await?;
        let second = subscriber.latest();

        assert_eq!(first.value.sequence, second.value.sequence);
        assert_eq!(first.value.changes.len(), second.value.changes.len());
        assert_eq!(first.value.timeline.len(), second.value.timeline.len());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_records_incremental_version_changes() -> Result<(), crate::state::WorkerError>
    {
        let cfg = sample_runtime_config();
        let (cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(4)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let before = subscriber.latest().value.sequence;

        let mut updated_cfg = cfg.clone();
        updated_cfg.api.security.tls.mode = ApiTlsMode::Required;
        cfg_publisher
            .publish(updated_cfg, UnixMillis(3))
            .map_err(|err| WorkerError::Message(format!("cfg publish failed: {err}")))?;

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert!(latest.value.sequence > before);

        let config_events = latest
            .value
            .changes
            .iter()
            .filter(|event| matches!(event.domain, DebugDomain::Config))
            .collect::<Vec<_>>();
        assert!(!config_events.is_empty());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_does_not_record_ha_tick_only_changes(
    ) -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));

        let initial_ha = sample_ha_state();
        let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha.clone(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(3)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber: ha_subscriber.clone(),
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let before = subscriber.latest();
        let before_timeline_len = before.value.timeline.len();
        let before_sequence = before.value.sequence;

        let mut ha_bumped_tick = initial_ha.clone();
        ha_bumped_tick.tick = ha_bumped_tick.tick.saturating_add(1);
        ha_publisher
            .publish(ha_bumped_tick.clone(), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

        super::step_once(&mut ctx).await?;
        let after = subscriber.latest();
        assert_eq!(after.value.timeline.len(), before_timeline_len);
        assert_eq!(after.value.sequence, before_sequence);
        assert_eq!(after.value.ha.value.tick, ha_bumped_tick.tick);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_history_retention_trims_old_entries() -> Result<(), crate::state::WorkerError>
    {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.history_limit = 3;
        ctx.now = Box::new(|| Ok(UnixMillis(2)));

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert_eq!(latest.value.changes.len(), 3);
        assert_eq!(latest.value.timeline.len(), 3);
        Ok(())
    }
}


===== docs/tmp/verbose_extra_context/http-api-deep-summary.md =====
# HTTP API deep summary

This support note is only raw factual context for `docs/src/reference/http-api.md`.
Prefer exact endpoint names, methods, auth roles, request/response types, and status behavior from the code. Do not invent features such as authentication schemes beyond bearer role tokens and optional TLS.

Public API surface from `src/api/worker.rs` route matching:

- `POST /switchover`
- `DELETE /ha/switchover`
- `GET /ha/state`
- `GET /fallback/cluster`
- `POST /fallback/heartbeat`
- `GET /debug/snapshot`
- `GET /debug/verbose`
- `GET /debug/ui`

Authentication and authorization behavior:

- `src/api/worker.rs` uses bearer-token auth when role tokens are configured.
- If both read/admin tokens are absent, requests are allowed.
- If tokens are configured and the request has no bearer token, the result is `401 Unauthorized`.
- If an endpoint requires admin and the request presents only the read token, the result is `403 Forbidden`.
- Admin endpoints are:
  - `POST /switchover`
  - `POST /fallback/heartbeat`
  - `DELETE /ha/switchover`
- All other listed routes are read endpoints.
- Requests use the `Authorization: Bearer <token>` header.
- Optional or required TLS can be configured; TLS is not itself the authorization model.

Generic HTTP status/error conventions:

- Unknown routes return `404 Not Found` with body `not found`.
- Invalid JSON on JSON-taking endpoints returns `400 Bad Request` with a message like `invalid json: ...`.
- `ApiError::BadRequest` maps to `400 Bad Request`.
- `ApiError::DcsStore` maps to `503 Service Unavailable`.
- `ApiError::Internal` maps to `500 Internal Server Error`.
- Some read routes can also return `503 Service Unavailable` when snapshot state is unavailable.

Endpoint details:

1. `GET /ha/state`
   - Returns `200 OK`.
   - Response type is `HaStateResponse`.
   - Fields are:
     - `cluster_name: String`
     - `scope: String`
     - `self_member_id: String`
     - `leader: Option<String>`
     - `switchover_requested_by: Option<String>`
     - `member_count: usize`
     - `dcs_trust: DcsTrustResponse`
     - `ha_phase: HaPhaseResponse`
     - `ha_tick: u64`
     - `ha_decision: HaDecisionResponse`
     - `snapshot_sequence: u64`
   - `DcsTrustResponse` values: `full_quorum`, `fail_safe`, `not_trusted`.
   - `HaPhaseResponse` values: `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, `fail_safe`.
   - `HaDecisionResponse` is a tagged enum with variants for no-change, waiting, leadership, follow, become-primary, step-down, replica recovery, fencing, leader-lease release, and fail-safe entry.
   - If no debug snapshot subscriber is attached, the route returns `503 Service Unavailable` with body `snapshot unavailable`.

2. `POST /switchover`
   - Requires admin authorization.
   - Request type is `SwitchoverRequestInput { requested_by: MemberId }`.
   - Unknown fields are denied by serde.
   - Empty or whitespace-only `requested_by` returns `400 Bad Request`.
   - On success, writes a typed switchover request to `/{scope}/switchover` in DCS and returns `202 Accepted` with `AcceptedResponse { accepted: true }`.

3. `DELETE /ha/switchover`
   - Requires admin authorization.
   - Clears the switchover request through the DCS HA writer.
   - Returns `202 Accepted` with `AcceptedResponse { accepted: true }` on success.

4. `GET /fallback/cluster`
   - Read authorization.
   - Returns `200 OK`.
   - Response type is `FallbackClusterView { name: String }`, where `name` is the configured cluster name.

5. `POST /fallback/heartbeat`
   - Requires admin authorization.
   - Request type is `FallbackHeartbeatInput { source: String }`.
   - Unknown fields are denied by serde.
   - Empty or whitespace-only `source` returns `400 Bad Request`.
   - Success returns `202 Accepted` with `AcceptedResponse { accepted: true }`.

6. `GET /debug/snapshot`
   - Read authorization.
   - Only available when `cfg.debug.enabled` is true.
   - If debug is disabled, returns `404 Not Found`.
   - If snapshot subscriber is missing, returns `503 Service Unavailable` with body `snapshot unavailable`.
   - On success returns `200 OK` and emits a pretty `Debug` string representation of the full system snapshot, not a stable JSON schema.

7. `GET /debug/verbose`
   - Read authorization.
   - Only available when `cfg.debug.enabled` is true.
   - If debug is disabled, returns `404 Not Found`.
   - Optional query parameter: `since=<u64>`.
   - Invalid `since` parsing returns `400 Bad Request`.
   - On success returns `200 OK` with a structured JSON payload assembled by `build_verbose_payload`.
   - That payload includes:
     - `meta`
     - `config`
     - `pginfo`
     - `dcs`
     - `process`
     - `ha`
     - `api`
     - `debug`
     - `changes`
     - `timeline`
   - The `api.endpoints` list in the payload is:
     - `/debug/snapshot`
     - `/debug/verbose`
     - `/debug/ui`
     - `/fallback/cluster`
     - `/switchover`
     - `/ha/state`
     - `/ha/switchover`

8. `GET /debug/ui`
   - Read authorization.
   - Only available when `cfg.debug.enabled` is true.
   - If debug is disabled, returns `404 Not Found`.
   - Returns `200 OK` with HTML content for the built-in debug UI.

Important docs caveats:

- `GET /debug/snapshot` returns debug-formatted text, not a documented JSON schema.
- The route list in `api.endpoints` is a helpful summary surface, but the authoritative behavior is the `route_request` match in `src/api/worker.rs`.
- There is no source-backed evidence in these files for additional auth mechanisms beyond bearer role tokens plus optional TLS/client-cert handling.
