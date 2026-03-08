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

docs/src/reference/runtime-configuration.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/how-to/check-cluster-health.md
docs/src/reference/pgtuskmasterctl-cli.md
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

# Reference

- [Reference]()
    - [pgtuskmasterctl CLI](reference/pgtuskmasterctl-cli.md)



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
docs/draft/docs/src/how-to/check-cluster-health.md
docs/draft/docs/src/how-to/check-cluster-health.revised.md
docs/draft/docs/src/reference/cli-commands.md
docs/draft/docs/src/reference/cli-commands.revised.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.revised.md
docs/draft/docs/src/reference/cli.md
docs/draft/docs/src/reference/cli.revised.md
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
docs/src/how-to/check-cluster-health.md
docs/src/reference/pgtuskmasterctl-cli.md
docs/src/tutorial/first-ha-cluster.md
docs/tmp/docs/src/explanation/architecture.prompt.md
docs/tmp/docs/src/how-to/check-cluster-health.prompt.md
docs/tmp/docs/src/reference/cli-commands.prompt.md
docs/tmp/docs/src/reference/cli-pgtuskmasterctl.prompt.md
docs/tmp/docs/src/reference/cli.prompt.md
docs/tmp/docs/src/reference/pgtuskmasterctl-cli.prompt.md
docs/tmp/docs/src/reference/runtime-configuration.prompt.md
docs/tmp/docs/src/tutorial/first-ha-cluster.prompt.md
docs/tmp/k2-batch/20260308-architecture.prepare.out
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
docs/tmp/verbose_extra_context/runtime-config-deep-summary.md
docs/tmp/verbose_extra_context/runtime-config-summary.md


===== src/config/schema.rs =====
use std::{collections::BTreeMap, fmt, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigVersion {
    V1,
    V2,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SecretSource(pub InlineOrPath);

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            InlineOrPath::Path(path) => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::PathConfig { path } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::Inline { .. } => f
                .debug_tuple("SecretSource")
                .field(&"<inline redacted>")
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiTlsMode {
    Disabled,
    Optional,
    Required,
}

pub type TlsMode = ApiTlsMode;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfig {
    pub cert_chain: InlineOrPath,
    pub private_key: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientAuthConfig {
    pub client_ca: InlineOrPath,
    pub require_client_cert: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfig {
    pub mode: TlsMode,
    pub identity: Option<TlsServerIdentityConfig>,
    pub client_auth: Option<TlsClientAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: String,
    pub member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: u32,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: PostgresConnIdentityConfig,
    pub rewind_conn_identity: PostgresConnIdentityConfig,
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub pg_hba: PgHbaConfig,
    pub pg_ident: PgIdentConfig,
    pub extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfig {
    pub user: String,
    pub dbname: String,
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfig {
    Tls,
    Password { password: SecretSource },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<String>,
    pub scope: String,
    pub init: Option<DcsInitConfig>,
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
    pub loop_interval_ms: u64,
    pub lease_ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    pub pg_rewind_timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub fencing_timeout_ms: u64,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub capture_subprocess_output: bool,
    pub postgres: PostgresLoggingConfig,
    pub sinks: LoggingSinksConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresLoggingConfig {
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    pub cleanup: LogCleanupConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingSinksConfig {
    pub stderr: StderrSinkConfig,
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrSinkConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub mode: FileSinkMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSinkMode {
    Append,
    Truncate,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCleanupConfig {
    pub enabled: bool,
    pub max_files: u64,
    pub max_age_seconds: u64,
    #[serde(default = "default_log_cleanup_protect_recent_seconds")]
    pub protect_recent_seconds: u64,
}

fn default_log_cleanup_protect_recent_seconds() -> u64 {
    300
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPaths {
    pub postgres: PathBuf,
    pub pg_ctl: PathBuf,
    pub pg_rewind: PathBuf,
    pub initdb: PathBuf,
    pub pg_basebackup: PathBuf,
    pub psql: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: String,
    pub security: ApiSecurityConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfig {
    pub tls: TlsServerConfig,
    pub auth: ApiAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAuthConfig {
    Disabled,
    RoleTokens(ApiRoleTokensConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiRoleTokensConfig {
    pub read_token: Option<String>,
    pub admin_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialRuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PartialPostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: PartialProcessConfig,
    pub logging: Option<PartialLoggingConfig>,
    pub api: Option<PartialApiConfig>,
    pub debug: Option<PartialDebugConfig>,
    pub security: Option<PartialSecurityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: Option<String>,
    pub listen_port: Option<u16>,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialProcessConfig {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingConfig {
    pub level: Option<LogLevel>,
    pub capture_subprocess_output: Option<bool>,
    pub postgres: Option<PartialPostgresLoggingConfig>,
    pub sinks: Option<PartialLoggingSinksConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresLoggingConfig {
    pub enabled: Option<bool>,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: Option<u64>,
    pub cleanup: Option<PartialLogCleanupConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLogCleanupConfig {
    pub enabled: Option<bool>,
    pub max_files: Option<u64>,
    pub max_age_seconds: Option<u64>,
    pub protect_recent_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingSinksConfig {
    pub stderr: Option<PartialStderrSinkConfig>,
    pub file: Option<PartialFileSinkConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialStderrSinkConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialFileSinkConfig {
    pub enabled: Option<bool>,
    pub path: Option<PathBuf>,
    pub mode: Option<FileSinkMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialApiConfig {
    pub listen_addr: Option<String>,
    pub read_auth_token: Option<String>,
    pub admin_auth_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialDebugConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialSecurityConfig {
    pub tls_enabled: Option<bool>,
    pub auth_token: Option<String>,
}

// -------------------------------
// v2 input schema (explicit secure)
// -------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigV2Input {
    pub config_version: ConfigVersion,
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfigV2Input,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfigV2Input,
    pub logging: Option<LoggingConfig>,
    pub api: ApiConfigV2Input,
    pub debug: Option<DebugConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfigV2Input {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: Option<BinaryPathsV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathsV2Input {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfigV2Input {
    pub listen_addr: Option<String>,
    pub security: Option<ApiSecurityConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfigV2Input {
    pub tls: Option<TlsServerConfigV2Input>,
    pub auth: Option<ApiAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfigV2Input {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub rewind_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub tls: Option<TlsServerConfigV2Input>,
    pub roles: Option<PostgresRolesConfigV2Input>,
    pub pg_hba: Option<PgHbaConfigV2Input>,
    pub pg_ident: Option<PgIdentConfigV2Input>,
    pub extra_gucs: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfigV2Input {
    pub user: Option<String>,
    pub dbname: Option<String>,
    pub ssl_mode: Option<crate::pginfo::conninfo::PgSslMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfigV2Input {
    pub username: Option<String>,
    pub auth: Option<RoleAuthConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfigV2Input {
    pub superuser: Option<PostgresRoleConfigV2Input>,
    pub replicator: Option<PostgresRoleConfigV2Input>,
    pub rewinder: Option<PostgresRoleConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfigV2Input {
    pub cert_chain: Option<InlineOrPath>,
    pub private_key: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfigV2Input {
    Tls,
    Password { password: Option<SecretSource> },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfigV2Input {
    pub mode: Option<TlsMode>,
    pub identity: Option<TlsServerIdentityConfigV2Input>,
    pub client_auth: Option<TlsClientAuthConfig>,
}


===== src/config/defaults.rs =====
use super::schema::{
    BinaryPaths, BinaryPathsV2Input, DebugConfig, FileSinkConfig, FileSinkMode, LogCleanupConfig,
    LogLevel, LoggingConfig, LoggingSinksConfig, PostgresLoggingConfig, ProcessConfig,
    StderrSinkConfig,
};
use super::ConfigError;

// This module is intentionally restricted to *safe* defaults only.
// It must not synthesize security-sensitive material (users/roles/auth, TLS posture, pg_hba/pg_ident).

const DEFAULT_PG_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;

const DEFAULT_API_LISTEN_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DEBUG_ENABLED: bool = false;

const DEFAULT_LOGGING_LEVEL: LogLevel = LogLevel::Info;
const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;
const DEFAULT_LOGGING_SINK_FILE_MODE: FileSinkMode = FileSinkMode::Append;

pub(crate) fn default_postgres_connect_timeout_s() -> u32 {
    DEFAULT_PG_CONNECT_TIMEOUT_S
}

pub(crate) fn default_api_listen_addr() -> String {
    DEFAULT_API_LISTEN_ADDR.to_string()
}

pub(crate) fn default_debug_config() -> DebugConfig {
    DebugConfig {
        enabled: DEFAULT_DEBUG_ENABLED,
    }
}

pub(crate) fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: DEFAULT_LOGGING_LEVEL,
        capture_subprocess_output: DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT,
        postgres: PostgresLoggingConfig {
            enabled: DEFAULT_LOGGING_POSTGRES_ENABLED,
            pg_ctl_log_file: None,
            log_dir: None,
            poll_interval_ms: DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
            cleanup: LogCleanupConfig {
                enabled: DEFAULT_LOGGING_CLEANUP_ENABLED,
                max_files: DEFAULT_LOGGING_CLEANUP_MAX_FILES,
                max_age_seconds: DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS,
                protect_recent_seconds: DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
            },
        },
        sinks: LoggingSinksConfig {
            stderr: StderrSinkConfig {
                enabled: DEFAULT_LOGGING_SINK_STDERR_ENABLED,
            },
            file: FileSinkConfig {
                enabled: DEFAULT_LOGGING_SINK_FILE_ENABLED,
                path: None,
                mode: DEFAULT_LOGGING_SINK_FILE_MODE,
            },
        },
    }
}

pub(crate) fn normalize_process_config(
    input: super::schema::ProcessConfigV2Input,
) -> Result<ProcessConfig, ConfigError> {
    let binaries = input.binaries.ok_or_else(|| ConfigError::Validation {
        field: "process.binaries",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    let binaries = normalize_binary_paths_v2(binaries)?;

    Ok(ProcessConfig {
        pg_rewind_timeout_ms: input
            .pg_rewind_timeout_ms
            .unwrap_or(DEFAULT_PG_REWIND_TIMEOUT_MS),
        bootstrap_timeout_ms: input
            .bootstrap_timeout_ms
            .unwrap_or(DEFAULT_BOOTSTRAP_TIMEOUT_MS),
        fencing_timeout_ms: input
            .fencing_timeout_ms
            .unwrap_or(DEFAULT_FENCING_TIMEOUT_MS),
        binaries,
    })
}

fn normalize_binary_paths_v2(input: BinaryPathsV2Input) -> Result<BinaryPaths, ConfigError> {
    Ok(BinaryPaths {
        postgres: require_binary_path("process.binaries.postgres", input.postgres)?,
        pg_ctl: require_binary_path("process.binaries.pg_ctl", input.pg_ctl)?,
        pg_rewind: require_binary_path("process.binaries.pg_rewind", input.pg_rewind)?,
        initdb: require_binary_path("process.binaries.initdb", input.initdb)?,
        pg_basebackup: require_binary_path("process.binaries.pg_basebackup", input.pg_basebackup)?,
        psql: require_binary_path("process.binaries.psql", input.psql)?,
    })
}

fn require_binary_path(
    field: &'static str,
    value: Option<std::path::PathBuf>,
) -> Result<std::path::PathBuf, ConfigError> {
    value.ok_or_else(|| ConfigError::Validation {
        field,
        message: "missing required secure field for config_version=v2".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_logging_config_is_deterministic() {
        let a = default_logging_config();
        let b = default_logging_config();
        assert_eq!(a, b);
    }
}


===== src/config/parser.rs =====
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::defaults::{
    default_api_listen_addr, default_debug_config, default_logging_config,
    default_postgres_connect_timeout_s, normalize_process_config,
};
use super::schema::{
    ApiConfig, ApiSecurityConfig, ConfigVersion, InlineOrPath, PgHbaConfig, PgIdentConfig,
    PostgresConfig, PostgresConnIdentityConfig, PostgresRoleConfig, PostgresRolesConfig,
    RoleAuthConfig, RoleAuthConfigV2Input, RuntimeConfig, RuntimeConfigV2Input, SecretSource,
    TlsServerConfig, TlsServerIdentityConfig,
};
use crate::postgres_managed_conf::{validate_extra_guc_entry, ManagedPostgresConfError};

const MIN_TIMEOUT_MS: u64 = 1;
const MAX_TIMEOUT_MS: u64 = 86_400_000;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config field `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;

    #[derive(serde::Deserialize)]
    struct ConfigEnvelope {
        config_version: Option<ConfigVersion>,
    }

    let envelope: ConfigEnvelope =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    let config_version = envelope.config_version.ok_or_else(|| ConfigError::Validation {
        field: "config_version",
        message: "missing required field; set config_version = \"v2\" to use the explicit secure schema".to_string(),
    })?;

    match config_version {
        ConfigVersion::V1 => {
            probe_legacy_v1_shape_for_diagnostics(&contents);
            Err(ConfigError::Validation {
                field: "config_version",
                message: "config_version = \"v1\" is no longer supported because it depends on implicit security defaults; migrate to config_version = \"v2\""
                    .to_string(),
            })
        }
        ConfigVersion::V2 => {
            let raw: RuntimeConfigV2Input =
                toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            let cfg = normalize_runtime_config_v2(raw)?;
            validate_runtime_config(&cfg)?;
            Ok(cfg)
        }
    }
}

fn probe_legacy_v1_shape_for_diagnostics(contents: &str) {
    // We keep the legacy v1 deserialization surface "alive" to:
    // - avoid unused-schema drift during the transition
    // - allow future improvements that surface rich TOML diagnostics for v1 migrations
    //
    // This must never override the v1 migration guidance with a parse error.
    let parsed: Result<toml::Value, toml::de::Error> = toml::from_str(contents);
    let Ok(mut value) = parsed else {
        return;
    };

    let Some(table) = value.as_table_mut() else {
        return;
    };

    let _ = table.remove("config_version");

    let _: Result<super::schema::PartialRuntimeConfig, toml::de::Error> = value.try_into();
}

fn normalize_runtime_config_v2(input: RuntimeConfigV2Input) -> Result<RuntimeConfig, ConfigError> {
    if !matches!(input.config_version, ConfigVersion::V2) {
        return Err(ConfigError::Validation {
            field: "config_version",
            message: "expected config_version = \"v2\"".to_string(),
        });
    }

    let postgres = normalize_postgres_config_v2(input.postgres)?;
    let process = normalize_process_config(input.process)?;
    let logging = input.logging.unwrap_or_else(default_logging_config);
    let api = normalize_api_config_v2(input.api)?;
    let debug = input.debug.unwrap_or_else(default_debug_config);

    Ok(RuntimeConfig {
        cluster: input.cluster,
        postgres,
        dcs: input.dcs,
        ha: input.ha,
        process,
        logging,
        api,
        debug,
    })
}

fn normalize_postgres_config_v2(
    input: super::schema::PostgresConfigV2Input,
) -> Result<PostgresConfig, ConfigError> {
    let connect_timeout_s = input
        .connect_timeout_s
        .unwrap_or_else(default_postgres_connect_timeout_s);

    let local_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.local_conn_identity",
        input.local_conn_identity,
    )?;
    let rewind_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.rewind_conn_identity",
        input.rewind_conn_identity,
    )?;

    let tls = normalize_tls_server_config_v2("postgres.tls", input.tls)?;
    let roles = normalize_postgres_roles_v2(input.roles)?;
    let pg_hba = normalize_pg_hba_v2(input.pg_hba)?;
    let pg_ident = normalize_pg_ident_v2(input.pg_ident)?;

    Ok(PostgresConfig {
        data_dir: input.data_dir,
        connect_timeout_s,
        listen_host: input.listen_host,
        listen_port: input.listen_port,
        socket_dir: input.socket_dir,
        log_file: input.log_file,
        local_conn_identity,
        rewind_conn_identity,
        tls,
        roles,
        pg_hba,
        pg_ident,
        extra_gucs: normalize_postgres_extra_gucs_v2(input.extra_gucs)?,
    })
}

fn normalize_postgres_extra_gucs_v2(
    input: Option<std::collections::BTreeMap<String, String>>,
) -> Result<std::collections::BTreeMap<String, String>, ConfigError> {
    let extra_gucs = input.unwrap_or_default();
    for (key, value) in &extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }
    Ok(extra_gucs)
}

fn normalize_postgres_conn_identity_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresConnIdentityConfigV2Input>,
) -> Result<PostgresConnIdentityConfig, ConfigError> {
    let identity = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let user_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.user",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.user",
        _ => field_prefix,
    };
    let dbname_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.dbname",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.dbname",
        _ => field_prefix,
    };
    let ssl_mode_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.ssl_mode",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.ssl_mode",
        _ => field_prefix,
    };

    let user = identity.user.ok_or_else(|| ConfigError::Validation {
        field: user_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(user_field, user.as_str())?;

    let dbname = identity.dbname.ok_or_else(|| ConfigError::Validation {
        field: dbname_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(dbname_field, dbname.as_str())?;

    let ssl_mode = identity.ssl_mode.ok_or_else(|| ConfigError::Validation {
        field: ssl_mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(PostgresConnIdentityConfig {
        user,
        dbname,
        ssl_mode,
    })
}

fn normalize_postgres_roles_v2(
    input: Option<super::schema::PostgresRolesConfigV2Input>,
) -> Result<PostgresRolesConfig, ConfigError> {
    let roles = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.roles",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let superuser = normalize_postgres_role_v2("postgres.roles.superuser", roles.superuser)?;
    let replicator = normalize_postgres_role_v2("postgres.roles.replicator", roles.replicator)?;
    let rewinder = normalize_postgres_role_v2("postgres.roles.rewinder", roles.rewinder)?;

    Ok(PostgresRolesConfig {
        superuser,
        replicator,
        rewinder,
    })
}

fn normalize_postgres_role_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresRoleConfigV2Input>,
) -> Result<PostgresRoleConfig, ConfigError> {
    let role = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let username_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.username",
        "postgres.roles.replicator" => "postgres.roles.replicator.username",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.username",
        _ => field_prefix,
    };
    let auth_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.auth",
        "postgres.roles.replicator" => "postgres.roles.replicator.auth",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.auth",
        _ => field_prefix,
    };

    let username = role.username.ok_or_else(|| ConfigError::Validation {
        field: username_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(username_field, username.as_str())?;

    let auth = role.auth.ok_or_else(|| ConfigError::Validation {
        field: auth_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let auth = normalize_role_auth_config_v2(auth_field, auth)?;

    Ok(PostgresRoleConfig { username, auth })
}

fn normalize_role_auth_config_v2(
    field_prefix: &'static str,
    input: RoleAuthConfigV2Input,
) -> Result<RoleAuthConfig, ConfigError> {
    match input {
        RoleAuthConfigV2Input::Tls => Ok(RoleAuthConfig::Tls),
        RoleAuthConfigV2Input::Password { password } => {
            let password_field = match field_prefix {
                "postgres.roles.superuser.auth" => "postgres.roles.superuser.auth.password",
                "postgres.roles.replicator.auth" => "postgres.roles.replicator.auth.password",
                "postgres.roles.rewinder.auth" => "postgres.roles.rewinder.auth.password",
                _ => field_prefix,
            };

            let password = password.ok_or_else(|| ConfigError::Validation {
                field: password_field,
                message: "missing required secure field for config_version=v2".to_string(),
            })?;

            Ok(RoleAuthConfig::Password { password })
        }
    }
}

fn normalize_pg_hba_v2(
    input: Option<super::schema::PgHbaConfigV2Input>,
) -> Result<PgHbaConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgHbaConfig { source })
}

fn normalize_pg_ident_v2(
    input: Option<super::schema::PgIdentConfigV2Input>,
) -> Result<PgIdentConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgIdentConfig { source })
}

fn normalize_api_config_v2(
    input: super::schema::ApiConfigV2Input,
) -> Result<ApiConfig, ConfigError> {
    let listen_addr = input.listen_addr.unwrap_or_else(default_api_listen_addr);

    let security = input.security.ok_or_else(|| ConfigError::Validation {
        field: "api.security",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let tls = normalize_tls_server_config_v2("api.security.tls", security.tls)?;
    let auth = security.auth.ok_or_else(|| ConfigError::Validation {
        field: "api.security.auth",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(ApiConfig {
        listen_addr,
        security: ApiSecurityConfig { tls, auth },
    })
}

fn normalize_tls_server_config_v2(
    field_prefix: &'static str,
    input: Option<super::schema::TlsServerConfigV2Input>,
) -> Result<TlsServerConfig, ConfigError> {
    let tls = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let mode_field = match field_prefix {
        "postgres.tls" => "postgres.tls.mode",
        "api.security.tls" => "api.security.tls.mode",
        _ => field_prefix,
    };
    let identity_field = match field_prefix {
        "postgres.tls" => "postgres.tls.identity",
        "api.security.tls" => "api.security.tls.identity",
        _ => field_prefix,
    };

    let mode = tls.mode.ok_or_else(|| ConfigError::Validation {
        field: mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let identity = match tls.identity {
        None => None,
        Some(identity) => Some(normalize_tls_server_identity_v2(identity_field, identity)?),
    };

    Ok(TlsServerConfig {
        mode,
        identity,
        client_auth: tls.client_auth,
    })
}

fn normalize_tls_server_identity_v2(
    field_prefix: &'static str,
    input: super::schema::TlsServerIdentityConfigV2Input,
) -> Result<TlsServerIdentityConfig, ConfigError> {
    let cert_chain_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.cert_chain",
        "api.security.tls.identity" => "api.security.tls.identity.cert_chain",
        _ => field_prefix,
    };
    let private_key_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.private_key",
        "api.security.tls.identity" => "api.security.tls.identity.private_key",
        _ => field_prefix,
    };

    let cert_chain = input.cert_chain.ok_or_else(|| ConfigError::Validation {
        field: cert_chain_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    let private_key = input.private_key.ok_or_else(|| ConfigError::Validation {
        field: private_key_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(TlsServerIdentityConfig {
        cert_chain,
        private_key,
    })
}

fn validate_absolute_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if !path.is_absolute() {
        return Err(ConfigError::Validation {
            field,
            message: "must be an absolute path".to_string(),
        });
    }
    Ok(())
}

fn normalize_path_lexical(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty_path("postgres.data_dir", &cfg.postgres.data_dir)?;
    validate_non_empty("postgres.listen_host", cfg.postgres.listen_host.as_str())?;
    validate_port("postgres.listen_port", cfg.postgres.listen_port)?;
    validate_non_empty_path("postgres.socket_dir", &cfg.postgres.socket_dir)?;
    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;

    validate_non_empty(
        "postgres.local_conn_identity.user",
        cfg.postgres.local_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.local_conn_identity.dbname",
        cfg.postgres.local_conn_identity.dbname.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.user",
        cfg.postgres.rewind_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.dbname",
        cfg.postgres.rewind_conn_identity.dbname.as_str(),
    )?;

    validate_non_empty(
        "postgres.roles.superuser.username",
        cfg.postgres.roles.superuser.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.replicator.username",
        cfg.postgres.roles.replicator.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.rewinder.username",
        cfg.postgres.roles.rewinder.username.as_str(),
    )?;

    if cfg.postgres.local_conn_identity.user != cfg.postgres.roles.superuser.username {
        return Err(ConfigError::Validation {
            field: "postgres.local_conn_identity.user",
            message: format!(
                "must match postgres.roles.superuser.username (got `{}`, expected `{}`)",
                cfg.postgres.local_conn_identity.user, cfg.postgres.roles.superuser.username
            ),
        });
    }
    if cfg.postgres.rewind_conn_identity.user != cfg.postgres.roles.rewinder.username {
        return Err(ConfigError::Validation {
            field: "postgres.rewind_conn_identity.user",
            message: format!(
                "must match postgres.roles.rewinder.username (got `{}`, expected `{}`)",
                cfg.postgres.rewind_conn_identity.user, cfg.postgres.roles.rewinder.username
            ),
        });
    }

    validate_postgres_auth_tls_invariants(cfg)?;

    validate_role_auth(
        "postgres.roles.superuser.auth.password.path",
        "postgres.roles.superuser.auth.password.content",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_role_auth(
        "postgres.roles.replicator.auth.password.path",
        "postgres.roles.replicator.auth.password.content",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_role_auth(
        "postgres.roles.rewinder.auth.password.path",
        "postgres.roles.rewinder.auth.password.content",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_tls_server_config(
        "postgres.tls.identity",
        "postgres.tls.identity.cert_chain",
        "postgres.tls.identity.private_key",
        &cfg.postgres.tls,
    )?;
    validate_tls_client_auth_config(
        "postgres.tls.client_auth",
        "postgres.tls.client_auth.client_ca",
        &cfg.postgres.tls,
    )?;

    validate_inline_or_path_non_empty(
        "postgres.pg_hba.source",
        &cfg.postgres.pg_hba.source,
        false,
    )?;
    validate_inline_or_path_non_empty(
        "postgres.pg_ident.source",
        &cfg.postgres.pg_ident.source,
        false,
    )?;
    for (key, value) in &cfg.postgres.extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }

    validate_non_empty_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_absolute_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_non_empty_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_absolute_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_non_empty_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_absolute_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_non_empty_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_absolute_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_non_empty_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_absolute_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_non_empty_path("process.binaries.psql", &cfg.process.binaries.psql)?;
    validate_absolute_path("process.binaries.psql", &cfg.process.binaries.psql)?;

    validate_timeout(
        "process.pg_rewind_timeout_ms",
        cfg.process.pg_rewind_timeout_ms,
    )?;
    validate_timeout(
        "process.bootstrap_timeout_ms",
        cfg.process.bootstrap_timeout_ms,
    )?;
    validate_timeout("process.fencing_timeout_ms", cfg.process.fencing_timeout_ms)?;

    validate_timeout(
        "logging.postgres.poll_interval_ms",
        cfg.logging.postgres.poll_interval_ms,
    )?;
    if let Some(path) = cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.pg_ctl_log_file", path)?;
        validate_absolute_path("logging.postgres.pg_ctl_log_file", path)?;
    }
    if let Some(path) = cfg.logging.postgres.log_dir.as_ref() {
        validate_non_empty_path("logging.postgres.log_dir", path)?;
        validate_absolute_path("logging.postgres.log_dir", path)?;
    }
    if cfg.logging.postgres.cleanup.enabled {
        if cfg.logging.postgres.cleanup.max_files == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_files",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.max_age_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_age_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.protect_recent_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.protect_recent_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
    }

    if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
        validate_non_empty_path("logging.sinks.file.path", path)?;
    }

    if cfg.logging.sinks.file.enabled && cfg.logging.sinks.file.path.is_none() {
        return Err(ConfigError::Validation {
            field: "logging.sinks.file.path",
            message: "must be configured when logging.sinks.file.enabled is true".to_string(),
        });
    }

    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;
    validate_absolute_path("postgres.log_file", &cfg.postgres.log_file)?;

    if cfg.logging.sinks.file.enabled {
        if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
            validate_absolute_path("logging.sinks.file.path", path)?;
        }
    }

    validate_logging_path_ownership_invariants(cfg)?;

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "must contain at least one endpoint".to_string(),
        });
    }

    for endpoint in &cfg.dcs.endpoints {
        if endpoint.trim().is_empty() {
            return Err(ConfigError::Validation {
                field: "dcs.endpoints",
                message: "must not contain empty endpoint values".to_string(),
            });
        }
    }

    if cfg.dcs.scope.trim().is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.scope",
            message: "must not be empty".to_string(),
        });
    }

    if cfg.ha.loop_interval_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.loop_interval_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms <= cfg.ha.loop_interval_ms {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than ha.loop_interval_ms".to_string(),
        });
    }

    match &cfg.api.security.auth {
        crate::config::ApiAuthConfig::Disabled => {}
        crate::config::ApiAuthConfig::RoleTokens(tokens) => {
            validate_optional_non_empty(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_deref(),
            )?;
            validate_optional_non_empty(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_deref(),
            )?;
            if tokens.read_token.is_none() && tokens.admin_token.is_none() {
                return Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens",
                    message: "at least one of read_token or admin_token must be configured"
                        .to_string(),
                });
            }
        }
    }

    validate_tls_server_config(
        "api.security.tls.identity",
        "api.security.tls.identity.cert_chain",
        "api.security.tls.identity.private_key",
        &cfg.api.security.tls,
    )?;
    validate_tls_client_auth_config(
        "api.security.tls.client_auth",
        "api.security.tls.client_auth.client_ca",
        &cfg.api.security.tls,
    )?;

    validate_dcs_init_config(cfg)?;

    Ok(())
}

fn validate_extra_guc_for_config(key: &str, value: &str) -> Result<(), ConfigError> {
    validate_extra_guc_entry(key, value).map_err(|err| match err {
        ManagedPostgresConfError::InvalidExtraGuc { key, message } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` invalid: {message}"),
        },
        ManagedPostgresConfError::ReservedExtraGuc { key } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` is reserved by pgtuskmaster"),
        },
        ManagedPostgresConfError::InvalidPrimarySlotName { slot, message } => {
            ConfigError::Validation {
                field: "postgres.extra_gucs",
                message: format!(
                    "unexpected replica slot validation while checking extra gucs `{slot}`: {message}"
                ),
            }
        }
    })
}

fn validate_logging_path_ownership_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(sink_path) = cfg.logging.sinks.file.path.as_ref() else {
        return Ok(());
    };
    if !cfg.logging.sinks.file.enabled {
        return Ok(());
    }

    let effective_pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        Some(path) => path,
        None => &cfg.postgres.log_file,
    };

    let sink_path = normalize_path_lexical(sink_path);
    let postgres_log_file = normalize_path_lexical(&cfg.postgres.log_file);
    let effective_pg_ctl_log_file = normalize_path_lexical(effective_pg_ctl_log_file);

    let tailed_files: [(&'static str, &PathBuf); 2] = [
        ("postgres.log_file", &postgres_log_file),
        (
            "logging.postgres.pg_ctl_log_file",
            &effective_pg_ctl_log_file,
        ),
    ];

    for (field, path) in tailed_files {
        if &sink_path == path {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: format!("must not equal tailed postgres input {field}"),
            });
        }
    }

    if let Some(log_dir) = cfg.logging.postgres.log_dir.as_ref() {
        let log_dir = normalize_path_lexical(log_dir);
        if sink_path.starts_with(&log_dir) {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: "must not be inside logging.postgres.log_dir (would self-ingest)"
                    .to_string(),
            });
        }
    }

    Ok(())
}

fn validate_non_empty_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_timeout(field: &'static str, value: u64) -> Result<(), ConfigError> {
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&value) {
        return Err(ConfigError::Validation {
            field,
            message: format!("must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS} ms"),
        });
    }
    Ok(())
}

fn validate_port(field: &'static str, value: u16) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::Validation {
            field,
            message: "must be greater than zero".to_string(),
        });
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ConfigError> {
    if let Some(raw) = value {
        if raw.trim().is_empty() {
            return Err(ConfigError::Validation {
                field,
                message: "must not be empty when configured".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_role_auth(
    password_path_field: &'static str,
    password_content_field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Ok(()),
        RoleAuthConfig::Password { password } => {
            validate_secret_source_non_empty(password_path_field, password_content_field, password)
        }
    }
}

fn validate_postgres_auth_tls_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_postgres_role_auth_supported(
        "postgres.roles.superuser.auth",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.replicator.auth",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.rewinder.auth",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.local_conn_identity.ssl_mode",
        cfg.postgres.local_conn_identity.ssl_mode,
        cfg.postgres.tls.mode,
    )?;
    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.rewind_conn_identity.ssl_mode",
        cfg.postgres.rewind_conn_identity.ssl_mode,
        cfg.postgres.tls.mode,
    )?;

    Ok(())
}

fn validate_postgres_role_auth_supported(
    field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Err(ConfigError::Validation {
            field,
            message:
                "postgresql role TLS client auth is not implemented; use type = \"password\" for now"
                    .to_string(),
        }),
        RoleAuthConfig::Password { .. } => Ok(()),
    }
}

fn validate_postgres_conn_identity_ssl_mode_supported(
    field: &'static str,
    ssl_mode: crate::pginfo::conninfo::PgSslMode,
    tls_mode: crate::config::ApiTlsMode,
) -> Result<(), ConfigError> {
    if matches!(tls_mode, crate::config::ApiTlsMode::Disabled)
        && postgres_ssl_mode_requires_server_tls(ssl_mode)
    {
        return Err(ConfigError::Validation {
            field,
            message: format!(
                "must not require server TLS when postgres.tls.mode is disabled (got `{}`)",
                ssl_mode.as_str()
            ),
        });
    }

    Ok(())
}

fn postgres_ssl_mode_requires_server_tls(ssl_mode: crate::pginfo::conninfo::PgSslMode) -> bool {
    matches!(
        ssl_mode,
        crate::pginfo::conninfo::PgSslMode::Require
            | crate::pginfo::conninfo::PgSslMode::VerifyCa
            | crate::pginfo::conninfo::PgSslMode::VerifyFull
    )
}

fn validate_tls_server_config(
    identity_field: &'static str,
    cert_chain_field: &'static str,
    private_key_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Ok(());
    }

    let identity = cfg
        .identity
        .as_ref()
        .ok_or_else(|| ConfigError::Validation {
            field: identity_field,
            message: "tls identity must be configured when tls.mode is optional or required"
                .to_string(),
        })?;

    validate_inline_or_path_non_empty(cert_chain_field, &identity.cert_chain, false)?;
    validate_inline_or_path_non_empty(private_key_field, &identity.private_key, false)?;
    Ok(())
}

fn validate_tls_client_auth_config(
    client_auth_field: &'static str,
    client_ca_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    let Some(client_auth) = cfg.client_auth.as_ref() else {
        return Ok(());
    };

    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Err(ConfigError::Validation {
            field: client_auth_field,
            message: "must not be configured when tls.mode is disabled".to_string(),
        });
    }

    validate_inline_or_path_non_empty(client_ca_field, &client_auth.client_ca, false)?;
    Ok(())
}

fn validate_dcs_init_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(init) = cfg.dcs.init.as_ref() else {
        return Ok(());
    };

    validate_non_empty("dcs.init.payload_json", init.payload_json.as_str())?;

    let _: serde_json::Value = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must be valid JSON: {err}"),
        }
    })?;

    let _: RuntimeConfig = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must decode as a RuntimeConfig JSON document: {err}"),
        }
    })?;

    Ok(())
}

fn validate_secret_source_non_empty(
    path_field: &'static str,
    content_field: &'static str,
    secret: &SecretSource,
) -> Result<(), ConfigError> {
    validate_inline_or_path_non_empty_for_secret(path_field, content_field, &secret.0)
}

fn validate_inline_or_path_non_empty_for_secret(
    path_field: &'static str,
    content_field: &'static str,
    value: &InlineOrPath,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(path_field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(path_field, path),
        InlineOrPath::Inline { content } => validate_non_empty(content_field, content.as_str()),
    }
}

fn validate_inline_or_path_non_empty(
    field: &'static str,
    value: &InlineOrPath,
    allow_empty_inline: bool,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(field, path),
        InlineOrPath::Inline { content } => {
            if allow_empty_inline {
                Ok(())
            } else {
                validate_non_empty(field, content.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
        ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig,
        InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
        PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig, PostgresLoggingConfig,
        PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
        StderrSinkConfig, TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: crate::config::SecretSource(crate::config::InlineOrPath::Inline {
                content: "secret-password".to_string(),
            }),
        }
    }

    fn expect_validation_error(
        result: Result<(), ConfigError>,
        expected_field: &'static str,
        expected_message_fragment: &str,
    ) -> Result<(), String> {
        match result {
            Err(ConfigError::Validation { field, message }) => {
                if field != expected_field {
                    return Err(format!(
                        "expected validation field {expected_field}, got {field}"
                    ));
                }
                if !message.contains(expected_message_fragment) {
                    return Err(format!(
                        "expected validation message to contain {expected_message_fragment:?}, got {message:?}"
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected validation error for {expected_field}, got {other:?}"
            )),
        }
    }

    fn base_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: sample_password_auth(),
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: sample_password_auth(),
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: sample_password_auth(),
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
                extra_gucs: std::collections::BTreeMap::new(),
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 120_000,
                bootstrap_timeout_ms: 300_000,
                fencing_timeout_ms: 30_000,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                    psql: PathBuf::from("/usr/bin/psql"),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: false },
        }
    }

    #[test]
    fn validate_runtime_config_accepts_valid_config() {
        let cfg = base_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_postgres_role_tls_auth() -> Result<(), String> {
        let mut superuser_cfg = base_runtime_config();
        superuser_cfg.postgres.roles.superuser.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&superuser_cfg),
            "postgres.roles.superuser.auth",
            "type = \"password\"",
        )?;

        let mut replicator_cfg = base_runtime_config();
        replicator_cfg.postgres.roles.replicator.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&replicator_cfg),
            "postgres.roles.replicator.auth",
            "type = \"password\"",
        )?;

        let mut rewinder_cfg = base_runtime_config();
        rewinder_cfg.postgres.roles.rewinder.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&rewinder_cfg),
            "postgres.roles.rewinder.auth",
            "type = \"password\"",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_local_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.local_conn_identity.ssl_mode = PgSslMode::Require;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.local_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_rewind_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.rewind_conn_identity.ssl_mode = PgSslMode::VerifyFull;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.rewind_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_empty_binary_path() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::new();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_non_absolute_binary_paths() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::from("pg_ctl");
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_bad_timeout() {
        let mut cfg = base_runtime_config();
        cfg.process.bootstrap_timeout_ms = 0;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.bootstrap_timeout_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_invalid_postgres_runtime_fields() {
        let mut cfg = base_runtime_config();
        cfg.postgres.listen_host = " ".to_string();
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_host",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.postgres.listen_port = 0;
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_port",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_missing_dcs_and_ha_invariants() {
        let mut cfg = base_runtime_config();
        cfg.dcs.endpoints.clear();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "dcs.endpoints",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.ha.lease_ttl_ms = cfg.ha.loop_interval_ms;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "ha.lease_ttl_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_blank_api_tokens() {
        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(" ".to_string()),
            admin_token: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.read_token",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: None,
            admin_token: Some("\t".to_string()),
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.admin_token",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_enabled_without_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_empty_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::new());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_accepts_file_sink_enabled_with_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster.jsonl"));

        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/./postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_parent_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/tmp/../postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_inside_log_dir_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.postgres.log_dir = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir"));
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir/./out.jsonl"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn load_runtime_config_missing_config_version_is_rejected(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_config_version_v1_is_rejected() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v1"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_unknown_fields_in_v2() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
connect_timeout_s = 5
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }
unknown = 10

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[logging]
level = "info"
capture_subprocess_output = true
postgres = { enabled = true, poll_interval_ms = 200, cleanup = { enabled = true, max_files = 10, max_age_seconds = 60 } }
sinks = { stderr = { enabled = true }, file = { enabled = false, mode = "append" } }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_happy_path_with_safe_defaults(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;
        let cfg = load_runtime_config(&path)?;
        assert_eq!(cfg.postgres.connect_timeout_s, 5);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 120_000);
        assert_eq!(cfg.process.bootstrap_timeout_ms, 300_000);
        assert_eq!(cfg.process.fencing_timeout_ms, 30_000);
        assert_eq!(cfg.api.listen_addr, "127.0.0.1:8080");
        assert!(!cfg.debug.enabled);

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_secure_fields_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-missing-{unique}.toml"));

        // Intentionally omit `postgres.local_conn_identity`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_process_binaries_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-v2-missing-binaries-{unique}.toml"));

        // Intentionally omit `process.binaries`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_password_auth_missing_password_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-auth-password-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.superuser.auth.password`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_postgres_roles_block_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-v2-missing-roles-{unique}.toml"));

        // Intentionally omit `postgres.roles`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_role_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-role-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_username_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-username-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.username`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.username",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_auth_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-auth-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.auth`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator" }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_conn_identity_role_mismatch(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-conn-identity-mismatch-{unique}.toml"
        ));

        // Intentionally set local_conn_identity.user to a different user than roles.superuser.username.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "not-postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity.user",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_blank_password_secret(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-blank-password-secret-{unique}.toml"
        ));

        // Intentionally set password secret content to empty.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password.content",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_tls_required_without_identity(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-required-tls-no-identity-{unique}.toml"
        ));

        // Intentionally omit `postgres.tls.identity` while requiring TLS.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "required" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_client_auth_with_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-client-auth-with-tls-disabled-{unique}.toml"
        ));

        // Intentionally configure client auth while TLS is disabled.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled", client_auth = { client_ca = { content = "client-ca" }, require_client_cert = false } }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.client_auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_postgres_role_tls_auth_with_actionable_error(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-postgres-role-tls-auth-{unique}.toml"
        ));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.roles.superuser.auth" {
                    Err(format!(
                        "expected validation field postgres.roles.superuser.auth, got {field}"
                    ))
                } else if !message.contains("type = \"password\"") {
                    Err(format!(
                        "expected validation message to mention password auth, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-postgres-ssl-mode-requires-tls-{unique}.toml"
        ));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.local_conn_identity.ssl_mode" {
                    Err(format!(
                        "expected validation field postgres.local_conn_identity.ssl_mode, got {field}"
                    ))
                } else if !message.contains("postgres.tls.mode is disabled") {
                    Err(format!(
                        "expected validation message to mention disabled postgres TLS, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
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


===== docker/configs/single/node-a/runtime.toml =====
config_version = "v2"

[cluster]
name = "docker-single"
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
scope = "docker-single"

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


===== docs/tmp/verbose_extra_context/runtime-config-deep-summary.md =====
# Runtime configuration deep summary

This note is a source-backed summary for `docs/src/reference/runtime-configuration.md`.

The runtime config entrypoint is `load_runtime_config(path)` in `src/config/parser.rs`.
It reads TOML text, parses a small envelope to extract `config_version`, and then only accepts `config_version = "v2"`.
If `config_version` is missing, parsing returns a validation error for `config_version` with guidance to set it to `v2`.
If `config_version = "v1"`, parsing rejects it explicitly and says v1 is no longer supported.

The top-level runtime config shape is `RuntimeConfig` in `src/config/schema.rs`.
It requires these top-level sections:
- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `debug`

For v2 input, some sections are structurally present but still allow defaults during normalization:
- `logging` is optional in `RuntimeConfigV2Input` and defaults via `default_logging_config()`
- `debug` is optional in `RuntimeConfigV2Input` and defaults via `default_debug_config()`
- `api.listen_addr` defaults to `127.0.0.1:8080`
- `postgres.connect_timeout_s` defaults to `5`
- `process.pg_rewind_timeout_ms` defaults to `120000`
- `process.bootstrap_timeout_ms` defaults to `300000`
- `process.fencing_timeout_ms` defaults to `30000`

Important secure-schema rule:
The parser intentionally requires explicit secure config blocks for fields like `postgres.local_conn_identity`, `postgres.roles`, `postgres.pg_hba`, `postgres.pg_ident`, `postgres.tls`, and `api.security`.
Missing these blocks yields `ConfigError::Validation` with field-specific messages saying the secure field or block is required for `config_version=v2`.

Top-level sections and important nested fields from `src/config/schema.rs`:

`cluster`
- `name: String`
- `member_id: String`

`postgres`
- `data_dir: PathBuf`
- `connect_timeout_s: u32`
- `listen_host: String`
- `listen_port: u16`
- `socket_dir: PathBuf`
- `log_file: PathBuf`
- `local_conn_identity`
- `rewind_conn_identity`
- `tls`
- `roles`
- `pg_hba`
- `pg_ident`
- `extra_gucs: BTreeMap<String, String>`

`postgres.local_conn_identity` and `postgres.rewind_conn_identity`
- `user`
- `dbname`
- `ssl_mode`

`postgres.roles`
- `superuser`
- `replicator`
- `rewinder`

Each role entry has:
- `username`
- `auth`

`auth` is tagged by `type` and currently supports:
- `tls`
- `password`

For `password`, the secret source is `SecretSource(InlineOrPath)`.
`InlineOrPath` supports:
- bare path
- `{ path = ... }`
- `{ content = ... }`

`postgres.tls` and `api.security.tls` use `TlsServerConfig`:
- `mode`
- `identity`
- `client_auth`

TLS mode enum values:
- `disabled`
- `optional`
- `required`

`dcs`
- `endpoints: Vec<String>`
- `scope: String`
- `init: Option<DcsInitConfig>`

`dcs.init`
- `payload_json`
- `write_on_bootstrap`

`ha`
- `loop_interval_ms`
- `lease_ttl_ms`

`process`
- `pg_rewind_timeout_ms`
- `bootstrap_timeout_ms`
- `fencing_timeout_ms`
- `binaries`

`process.binaries`
- `postgres`
- `pg_ctl`
- `pg_rewind`
- `initdb`
- `pg_basebackup`
- `psql`

`logging`
- `level`
- `capture_subprocess_output`
- `postgres`
- `sinks`

`logging.level` enum values:
- `trace`
- `debug`
- `info`
- `warn`
- `error`
- `fatal`

`logging.postgres`
- `enabled`
- `pg_ctl_log_file`
- `log_dir`
- `poll_interval_ms`
- `cleanup`

`logging.postgres.cleanup`
- `enabled`
- `max_files`
- `max_age_seconds`
- `protect_recent_seconds`

`logging.sinks.stderr`
- `enabled`

`logging.sinks.file`
- `enabled`
- `path`
- `mode`

`logging.sinks.file.mode` enum values:
- `append`
- `truncate`

`api`
- `listen_addr`
- `security`

`api.security.auth` supports:
- `disabled`
- `role_tokens`

When `role_tokens` is used, the config carries:
- `read_token`
- `admin_token`

`debug`
- `enabled`

Validation behavior from `validate_runtime_config(cfg)` and related helpers in `src/config/parser.rs`:
- path fields like `process.binaries.*` must be non-empty absolute paths
- timeout fields are checked with minimum and maximum bounds
- port fields are validated
- many string fields must be non-empty after trimming
- `postgres.local_conn_identity.user` must match `postgres.roles.superuser.username`
- `postgres.rewind_conn_identity.user` must match `postgres.roles.rewinder.username`
- postgres role auth and TLS settings are checked for supported combinations
- TLS identity and client-auth blocks are validated when TLS modes require them
- `postgres.pg_hba.source` and `postgres.pg_ident.source` must be non-empty
- `extra_gucs` keys and values are validated through `validate_extra_guc_entry`
- file sink and postgres logging paths are checked for ownership and overlap invariants
- DCS init JSON is parsed and validated

Concrete example observations from Docker runtime config examples:
- both `docker/configs/cluster/node-a/runtime.toml` and `docker/configs/single/node-a/runtime.toml` set `config_version = "v2"`
- both examples include every major section explicitly
- the examples set postgres TLS mode to `disabled`
- password-bearing role auth uses `{ path = "/run/secrets/..." }`
- both examples configure `logging.sinks.file.enabled = true`
- both examples set `api.security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }`
- both examples override `api.listen_addr` to `0.0.0.0:8080`, which is broader than the parser default

CLI interaction evidence from `cargo run --bin pgtuskmaster -- --help`:
- the daemon exposes `--config <PATH>`
- help text says it is the path to the runtime config file

Use this note as exhaustive factual support. Avoid inventing defaults or claiming a field is optional unless the parser or v2 input schema proves it.


===== docs/draft/docs/src/reference/runtime-configuration.md =====
