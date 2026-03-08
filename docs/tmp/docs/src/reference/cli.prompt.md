You are drafting exactly one documentation file.

Rules:
- Follow Diataxis strictly.
- Use only the supplied repo facts and supplied Diataxis summary.
- If a fact is missing, say "missing source support" instead of inventing.
- ASCII only.
- No em dashes.
- Prefer diagrams only when the supplied facts support every node and edge.

Behavior requirements:
- Read the target path and infer the intended page boundary from it.
- Use the Diataxis type that best matches the supplied target and evidence.
- Keep unsupported claims out of the document.
- If an important fact is missing, write "missing source support" at the exact point where that fact would otherwise be needed.

Follow Diataxis method, write one real page, and include diagrams when needed using the syntax:

[diagram about x, y showing relation between z and a, **more details on diagram**]


# target docs path

docs/src/reference/cli.md

# docs/src file listing

# docs/src file listing

docs/src/SUMMARY.md
docs/src/how-to/check-cluster-health.md
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
docs/draft/docs/src/how-to/check-cluster-health.md
docs/draft/docs/src/how-to/check-cluster-health.revised.md
docs/draft/docs/src/reference/cli-commands.md
docs/draft/docs/src/reference/cli-pgtuskmasterctl.md
docs/draft/docs/src/reference/cli.md
docs/draft/docs/src/reference/pgtuskmasterctl-cli.md
docs/draft/docs/src/reference/runtime-configuration.md
docs/draft/docs/src/tutorial/first-ha-cluster.final.md
docs/draft/docs/src/tutorial/first-ha-cluster.md
docs/draft/docs/src/tutorial/first-ha-cluster.revised.md
docs/mermaid-init.js
docs/mermaid.min.js
docs/src/SUMMARY.md
docs/src/how-to/check-cluster-health.md
docs/src/tutorial/first-ha-cluster.md
docs/tmp/docs/src/how-to/check-cluster-health.prompt.md
docs/tmp/docs/src/reference/cli-commands.prompt.md
docs/tmp/docs/src/reference/cli-pgtuskmasterctl.prompt.md
docs/tmp/docs/src/reference/cli.prompt.md
docs/tmp/docs/src/reference/pgtuskmasterctl-cli.prompt.md
docs/tmp/docs/src/reference/runtime-configuration.prompt.md
docs/tmp/docs/src/tutorial/first-ha-cluster.prompt.md
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
docs/tmp/verbose_extra_context/check-cluster-health-api-and-state.md
docs/tmp/verbose_extra_context/check-cluster-health-cli-overview.md
docs/tmp/verbose_extra_context/check-cluster-health-runtime-evidence.md
docs/tmp/verbose_extra_context/cli-surface-summary.md
docs/tmp/verbose_extra_context/cluster-start-command.md
docs/tmp/verbose_extra_context/leader-check-command.md
docs/tmp/verbose_extra_context/runtime-config-summary.md


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


===== src/cli/output.rs =====
use serde::Serialize;

use crate::cli::{
    args::OutputFormat,
    client::{AcceptedResponse, HaDecisionResponse, HaStateResponse},
    error::CliError,
    CommandOutput,
};

pub fn render_output(
    command_output: &CommandOutput,
    format: OutputFormat,
) -> Result<String, CliError> {
    match format {
        OutputFormat::Json => render_json(command_output),
        OutputFormat::Text => Ok(render_text(command_output)),
    }
}

fn render_json(command_output: &CommandOutput) -> Result<String, CliError> {
    #[derive(Serialize)]
    #[serde(untagged)]
    enum OutputRef<'a> {
        State(&'a HaStateResponse),
        Accepted(&'a AcceptedResponse),
    }

    let payload = match command_output {
        CommandOutput::HaState(value) => OutputRef::State(value.as_ref()),
        CommandOutput::Accepted(value) => OutputRef::Accepted(value),
    };

    serde_json::to_string_pretty(&payload)
        .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
}

fn render_text(command_output: &CommandOutput) -> String {
    match command_output {
        CommandOutput::Accepted(value) => format!("accepted={}", value.accepted),
        CommandOutput::HaState(value) => {
            let value = value.as_ref();
            let leader = value.leader.as_deref().unwrap_or("<none>");
            let switchover = value.switchover_requested_by.as_deref().unwrap_or("<none>");
            [
                format!("cluster_name={}", value.cluster_name),
                format!("scope={}", value.scope),
                format!("self_member_id={}", value.self_member_id),
                format!("leader={leader}"),
                format!("switchover_requested_by={switchover}"),
                format!("member_count={}", value.member_count),
                format!("dcs_trust={}", value.dcs_trust),
                format!("ha_phase={}", value.ha_phase),
                format!("ha_tick={}", value.ha_tick),
                format!("ha_decision={}", render_decision_text(&value.ha_decision)),
                format!("snapshot_sequence={}", value.snapshot_sequence),
            ]
            .join("\n")
        }
    }
}

fn render_decision_text(value: &HaDecisionResponse) -> String {
    match value {
        HaDecisionResponse::NoChange => "no_change".to_string(),
        HaDecisionResponse::WaitForPostgres {
            start_requested,
            leader_member_id,
        } => {
            let leader_detail = leader_member_id.as_deref().unwrap_or("none");
            format!(
                "wait_for_postgres(start_requested={start_requested}, leader_member_id={leader_detail})"
            )
        }
        HaDecisionResponse::WaitForDcsTrust => "wait_for_dcs_trust".to_string(),
        HaDecisionResponse::AttemptLeadership => "attempt_leadership".to_string(),
        HaDecisionResponse::FollowLeader { leader_member_id } => {
            format!("follow_leader(leader_member_id={leader_member_id})")
        }
        HaDecisionResponse::BecomePrimary { promote } => {
            format!("become_primary(promote={promote})")
        }
        HaDecisionResponse::StepDown {
            reason,
            release_leader_lease,
            clear_switchover,
            fence,
        } => format!(
            "step_down(reason={reason}, release_leader_lease={release_leader_lease}, clear_switchover={clear_switchover}, fence={fence})"
        ),
        HaDecisionResponse::RecoverReplica { strategy } => {
            format!("recover_replica(strategy={strategy})")
        }
        HaDecisionResponse::FenceNode => "fence_node".to_string(),
        HaDecisionResponse::ReleaseLeaderLease { reason } => {
            format!("release_leader_lease(reason={reason})")
        }
        HaDecisionResponse::EnterFailSafe {
            release_leader_lease,
        } => format!("enter_fail_safe(release_leader_lease={release_leader_lease})"),
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{
        args::OutputFormat,
        client::{AcceptedResponse, HaDecisionResponse, HaStateResponse},
        output::render_output,
        CommandOutput,
    };

    #[test]
    fn text_output_renders_state_lines() {
        let output = render_output(
            &CommandOutput::HaState(Box::new(HaStateResponse {
                cluster_name: "cluster-a".to_string(),
                scope: "scope-a".to_string(),
                self_member_id: "node-a".to_string(),
                leader: Some("node-a".to_string()),
                switchover_requested_by: None,
                member_count: 3,
                dcs_trust: crate::api::DcsTrustResponse::FullQuorum,
                ha_phase: crate::api::HaPhaseResponse::Primary,
                ha_tick: 9,
                ha_decision: HaDecisionResponse::BecomePrimary { promote: true },
                snapshot_sequence: 77,
            })),
            OutputFormat::Text,
        );
        assert!(output.is_ok(), "text render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered text should not be empty");
        assert!(rendered.contains("cluster_name=cluster-a"));
        assert!(rendered.contains("leader=node-a"));
        assert!(rendered.contains("switchover_requested_by=<none>"));
        assert!(rendered.contains("ha_decision=become_primary(promote=true)"));
    }

    #[test]
    fn json_output_renders_accepted_payload() {
        let output = render_output(
            &CommandOutput::Accepted(AcceptedResponse { accepted: true }),
            OutputFormat::Json,
        );
        assert!(output.is_ok(), "json render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered json should not be empty");
        assert!(rendered.contains("\"accepted\": true"));
    }
}


===== src/bin/pgtuskmasterctl.rs =====
use std::process::ExitCode;

use clap::Parser;
use pgtuskmaster_rust::cli::args::Cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match pgtuskmaster_rust::cli::run(cli).await {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            err.exit_code()
        }
    }
}


===== tests/cli_binary.rs =====
use std::process::Command;

fn write_temp_config(label: &str, toml: &str) -> Result<std::path::PathBuf, String> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system time error: {err}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "pgtuskmaster-cli-config-{label}-{unique}-{}",
        std::process::id()
    ));

    std::fs::write(&path, toml).map_err(|err| format!("write config failed: {err}"))?;
    Ok(path)
}

fn cli_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmasterctl") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmasterctl");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("cli binary not found at {}", candidate.display()))
    }
}

fn node_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmaster") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmaster");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("node binary not found at {}", candidate.display()))
    }
}

#[test]
fn help_exits_success() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("ha"),
        "help output should include ha command"
    );
    Ok(())
}

#[test]
fn missing_required_subcommand_arg_exits_usage_code() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .args(["ha", "leader", "set"])
        .output()
        .map_err(|err| format!("failed to run command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(2),
        "clap usage failures should exit with code 2"
    );
    Ok(())
}

#[test]
fn state_command_maps_connection_refused_to_exit_3() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("local_addr failed: {err}"))?;
    drop(listener);

    let output = Command::new(&bin)
        .args([
            "--base-url",
            &format!("http://{addr}"),
            "--timeout-ms",
            "50",
            "ha",
            "state",
        ])
        .output()
        .map_err(|err| format!("failed to run state command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(3),
        "transport errors should map to exit code 3"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("transport error"),
        "stderr should mention transport error"
    );
    Ok(())
}

#[test]
fn node_help_exits_success() -> Result<(), String> {
    let bin = node_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run node --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("--config"),
        "help output should include --config option"
    );
    Ok(())
}

#[test]
fn node_missing_config_version_prints_explicit_v2_migration_hint() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-config-version",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with missing config_version: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("set config_version = \"v2\""),
        "stderr should include explicit v2 migration hint, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_missing_secure_field_prints_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-process-binaries",
        r#"
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
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`process.binaries`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_postgres_role_tls_auth_with_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-role-tls-auth",
        r#"
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
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.roles.superuser.auth`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-ssl-mode-requires-tls",
        r#"
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
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.local_conn_identity.ssl_mode`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
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


===== docs/tmp/verbose_extra_context/cli-surface-summary.md =====
# Verbose CLI surface summary

This file exists to answer the "extra info" requests from the `choose-doc` outputs with source-derived facts only.

## Binary and entry point

- The operator CLI binary is `pgtuskmasterctl`.
- The binary entry point lives in `src/bin/pgtuskmasterctl.rs`.
- `main` parses `Cli` with clap, runs `pgtuskmaster_rust::cli::run(cli).await`, prints successful output to stdout, prints errors to stderr, and exits using the mapped CLI error exit code.

## Top-level CLI shape

The clap shape in `src/cli/args.rs` is:

- global flag: `--base-url <STRING>`
  - default: `http://127.0.0.1:8080`
- global flag: `--read-token <STRING>`
  - env fallback: `PGTUSKMASTER_READ_TOKEN`
- global flag: `--admin-token <STRING>`
  - env fallback: `PGTUSKMASTER_ADMIN_TOKEN`
- global flag: `--timeout-ms <U64>`
  - default: `5000`
- global flag: `--output <json|text>`
  - default: `json`
- top-level command group: `ha`

There are no other top-level command groups in `src/cli/args.rs` besides `ha`.

## HA subcommands

The `ha` command group contains:

- `ha state`
  - no subcommand-specific flags or positional arguments in `src/cli/args.rs`
  - implemented in `src/cli/mod.rs` by calling `CliApiClient::get_ha_state()`
- `ha switchover clear`
  - no extra flags
  - implemented in `src/cli/mod.rs` by calling `CliApiClient::delete_switchover()`
- `ha switchover request --requested-by <STRING>`
  - `--requested-by` is required
  - implemented in `src/cli/mod.rs` by calling `CliApiClient::post_switchover(requested_by)`

There is no `leader set` subcommand in the current clap tree. The integration test `tests/cli_binary.rs` intentionally invokes `ha leader set` as an invalid command and expects clap to exit with code `2`.

## Authentication behavior

- `--read-token` and `--admin-token` are both real CLI flags.
- Read operations use `read_token` first and fall back to `admin_token` if `read_token` is absent.
- Admin operations require `admin_token`.
- Empty or whitespace-only token values are normalized to `None` by `normalize_token` in `src/cli/client.rs`.

Role-to-command behavior from `src/cli/client.rs`:

- `get_ha_state` performs `GET /ha/state` with read-role auth
- `delete_switchover` performs `DELETE /ha/switchover` with admin-role auth
- `post_switchover` performs `POST /switchover` with admin-role auth and JSON body `{ "requested_by": "..." }`

Important factual note:

- The POST endpoint path used by the CLI client is `/switchover`, not `/ha/switchover`.
- The API controller code writes switchover state under a DCS path `/{scope}/switchover`.
- The delete path exposed by the CLI client is `/ha/switchover`.
- Any doc draft that claims both POST and DELETE use the same `/ha/switchover` API path should be checked carefully against source.

## Output formats

The only output formats are `json` and `text`.

For accepted/acknowledgement responses:

- JSON renders the serialized `AcceptedResponse`
- Text renders exactly `accepted=<bool>`

For `ha state` responses, JSON renders the `HaStateResponse` payload from `src/api/mod.rs` with these top-level fields:

- `cluster_name`
- `scope`
- `self_member_id`
- `leader`
- `switchover_requested_by`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_tick`
- `ha_decision`
- `snapshot_sequence`

Text mode renders these lines:

- `cluster_name=...`
- `scope=...`
- `self_member_id=...`
- `leader=...` or `<none>`
- `switchover_requested_by=...` or `<none>`
- `member_count=...`
- `dcs_trust=...`
- `ha_phase=...`
- `ha_tick=...`
- `ha_decision=...`
- `snapshot_sequence=...`

## HA state enums and payload shape

`dcs_trust` string values:

- `full_quorum`
- `fail_safe`
- `not_trusted`

`ha_phase` string values:

- `init`
- `waiting_postgres_reachable`
- `waiting_dcs_trusted`
- `waiting_switchover_successor`
- `replica`
- `candidate_leader`
- `primary`
- `rewinding`
- `bootstrapping`
- `fencing`
- `fail_safe`

`ha_decision` is tagged JSON with `kind` in snake_case. Variants in `src/api/mod.rs` are:

- `no_change`
- `wait_for_postgres`
  - fields: `start_requested`, `leader_member_id`
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader`
  - field: `leader_member_id`
- `become_primary`
  - field: `promote`
- `step_down`
  - fields: `reason`, `release_leader_lease`, `clear_switchover`, `fence`
- `recover_replica`
  - field: `strategy`
- `fence_node`
- `release_leader_lease`
  - field: `reason`
- `enter_fail_safe`
  - field: `release_leader_lease`

## Exit codes and observable behavior

From `src/cli/mod.rs` tests and the CLI error mapping:

- transport failures map to exit code `3`
- unexpected API status failures map to exit code `4`
- decode failures map to exit code `5`

From `tests/cli_binary.rs`:

- `pgtuskmasterctl --help` should succeed
- invalid clap usage should exit with code `2`
- unreachable `ha state` should exit with code `3` and mention `transport error`

