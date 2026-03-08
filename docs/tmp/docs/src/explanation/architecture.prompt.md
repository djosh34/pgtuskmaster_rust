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

docs/src/explanation/architecture.md

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


===== src/lib.rs =====
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod api;
pub mod cli;
pub mod config;
pub mod dcs;
pub(crate) mod debug_api;
pub(crate) mod ha;
pub(crate) mod logging;
pub mod pginfo;
pub(crate) mod postgres_managed;
pub(crate) mod postgres_managed_conf;
pub(crate) mod process;
pub mod runtime;
pub mod state;
#[doc(hidden)]
pub mod test_harness;
pub(crate) mod tls;

#[cfg(test)]
mod worker_contract_tests;


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


===== docs/tmp/verbose_extra_context/architecture-deep-summary.md =====
# Architecture deep summary

This note is a source-backed summary for `docs/src/explanation/architecture.md`.

The crate root in `src/lib.rs` exposes the major subsystems:
- `api`
- `cli`
- `config`
- `dcs`
- `runtime`
- `state`

There are also internal modules that matter to architecture:
- `ha`
- `logging`
- `postgres_managed`
- `postgres_managed_conf`
- `process`
- `tls`
- `debug_api`

The system is organized around a few cooperating concerns:
- runtime configuration loading
- PostgreSQL process management and state observation
- DCS state publication and trust evaluation
- HA decision making
- API projection of current cluster state

HA decision loop facts from `src/ha/decide.rs`:
- each decision tick derives `DecisionFacts` from the current world snapshot
- HA behavior is phase-driven
- main phases include `Init`, `WaitingPostgresReachable`, `WaitingDcsTrusted`, `Replica`, `CandidateLeader`, `Primary`, `Rewinding`, `Bootstrapping`, `Fencing`, `FailSafe`, and `WaitingSwitchoverSuccessor`
- the decision loop increments a tick counter and carries the last decision into the next state

Key safety behavior visible in `decide_phase` and related helpers:
- if DCS trust is not `FullQuorum`, the node does not continue normal leader logic
- if DCS trust is degraded and the local postgres is primary, the node enters `FailSafe` with `EnterFailSafe`
- if DCS trust is degraded and postgres is not primary, the node still enters `FailSafe` but with `NoChange`
- promotion only happens through explicit HA decisions such as `AttemptLeadership` followed by `BecomePrimary`
- when another active leader is detected while the local node is primary, the node enters `Fencing` and performs a `StepDown` plan with `fence = true`
- if primary postgres becomes unreachable while the node still holds leadership, the node releases the leader lease before recovery

Those transitions show the intended split-brain resistance:
- leadership depends on DCS trust
- the node treats foreign leaders as fencing conditions
- fail-safe mode interrupts normal leadership behavior when quorum trust is absent

DCS trust model from `src/dcs/state.rs`:
- trust values are `FullQuorum`, `FailSafe`, and `NotTrusted`
- if the etcd-backed store is unhealthy, trust becomes `NotTrusted`
- if the local member record is missing or stale, trust becomes `FailSafe`
- if a leader record exists but the leader member record is missing or stale, trust becomes `FailSafe`
- in multi-member caches, fewer than two fresh members also downgrades trust to `FailSafe`
- otherwise trust is `FullQuorum`

DCS cached state includes:
- member records
- leader record
- switchover request
- runtime config snapshot
- init lock

Member records carry enough replication state for HA comparisons:
- role
- SQL status
- readiness
- timeline
- write LSN
- replay LSN
- updated timestamp
- PostgreSQL version

API role from `src/api/controller.rs`:
- it writes switchover requests into the DCS namespace
- it deletes switchover requests through the DCS writer helper
- it maps internal HA and DCS state into stable API response enums and structs
- API responses expose cluster name, scope, self member id, leader, member count, DCS trust, HA phase, HA decision, and snapshot sequence

That means the API is mainly a control and observability surface, not the place where HA decisions are computed.
The controller translates internal state; it does not own the HA algorithm.

Operational invariant evidence from `tests/ha/support/observer.rs`:
- the HA observer samples API states and SQL roles over time
- it explicitly tracks `max_concurrent_primaries`
- it raises an error if more than one primary is observed
- it also rejects insufficient evidence windows when there are too few successful samples
- recent sample rings are retained to explain failures

This test harness behavior is useful architectural evidence:
- split-brain avoidance is a first-class invariant
- the system is expected to be judged over time-series observations, not just single snapshots
- API and SQL perspectives are both used to confirm safety

Concrete coordination story supported by the files:
- runtime config defines cluster identity, DCS scope, HA timeouts, postgres connection details, and API settings
- the DCS worker publishes and consumes cluster membership and leader information
- the HA worker consumes world facts built from DCS state and PostgreSQL reachability/role data
- the HA worker outputs decisions such as following a leader, attempting leadership, rewinding, bootstrapping, fencing, or entering fail-safe
- the API layer exposes the resulting state and lets operators request a switchover by writing into DCS

Be careful not to overclaim:
- the files here do not prove a full message-flow diagram or task scheduler topology beyond these module responsibilities
- the exact runtime thread model is not established by these excerpts alone
- any explanation should stay grounded in the visible trust evaluation, phase machine, and API translation responsibilities
