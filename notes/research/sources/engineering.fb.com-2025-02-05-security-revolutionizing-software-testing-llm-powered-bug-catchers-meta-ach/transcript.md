---
url: https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/
title: Revolutionizing software testing: Introducing LLM-powered bug catchers - Engineering at Meta
fetched: 2026-06-27
raw: raw.html
transport: curl
capture_status: ok
---

[Skip to content](#content) 

# [Engineering at Meta](https://engineering.fb.com/ "Engineering at Meta")

Search this site

![](https://engineering.fb.com/wp-content/themes/code-fb-com/img/icon-search.svg)

* [Open Source](# "Open Source") 
  + [Open Source](https://engineering.fb.com/category/open-source/ "Open Source")
  + [Meta Open Source](https://opensource.fb.com "Meta Open Source")
* [Platforms](# "Platforms") 
  + [Android](https://engineering.fb.com/category/android/ "Android")
  + [iOS](https://engineering.fb.com/category/ios/ "iOS")
  + [Web](https://engineering.fb.com/category/web/ "Web")
* [Infrastructure Systems](# "Infrastructure Systems") 
  + [Core Infra](https://engineering.fb.com/category/core-infra/ "Core Infra")
  + [Data Infrastructure](https://engineering.fb.com/category/data-infrastructure/ "Data Infrastructure")
  + [DevInfra](https://engineering.fb.com/category/developer-tools/ "DevInfra")
  + [Production Engineering](https://engineering.fb.com/category/production-engineering/ "Production Engineering")
  + [Security & Privacy](https://engineering.fb.com/category/security/ "Security & Privacy")
  + [Research Publications](https://research.facebook.com/publications/research-areas/systems-infrastructure/ "Research Publications")
* [Physical Infrastructure](# "Physical Infrastructure") 
  + [Connectivity](https://engineering.fb.com/category/connectivity/ "Connectivity")
  + [Data Center Engineering](https://engineering.fb.com/category/data-center-engineering/ "Data Center Engineering")
  + [Networking & Traffic](https://engineering.fb.com/category/networking-traffic/ "Networking & Traffic")
  + [Research Publications](https://research.facebook.com/publications/research-areas/networking-connectivity/ "Research Publications")
* [Video Engineering & AR/VR](# "Video Engineering & AR/VR") 
  + [Video Engineering](https://engineering.fb.com/category/video-engineering/ "Video Engineering")
  + [Virtual Reality](https://engineering.fb.com/category/virtual-reality/ "Virtual Reality")
  + [Research Publications](https://research.facebook.com/publications/research-areas/augmented-reality-virtual-reality/ "Research Publications")
* [Artificial Intelligence](# "Artificial Intelligence") 
  + [ML Applications](https://engineering.fb.com/category/ml-applications/ "ML Applications")
  + [AI Research](https://engineering.fb.com/category/ai-research/ "AI Research")
  + [Research Publications](https://ai.facebook.com/results/?content_types%5B0%5D=publication "Research Publications")
* [Watch Videos](/videos "Watch Videos")

POSTED ON FEBRUARY 5, 2025 TO [ML Applications](https://engineering.fb.com/category/ml-applications/), [Security & Privacy](https://engineering.fb.com/category/security/)

# Revolutionizing software testing: Introducing LLM-powered bug catchers

![](https://engineering.fb.com/wp-content/uploads/2021/10/RiB_DarkBlue.jpg "RiB_DarkBlue_Tile") 

By [Christopher Foster](https://engineering.fb.com/author/christopher-foster/ "Posts by Christopher Foster"), [Abhishek Gulati](https://engineering.fb.com/author/abhishek-gulati/ "Posts by Abhishek Gulati"), [Mark Harman](https://engineering.fb.com/author/mark-harman/ "Posts by Mark Harman"), [Inna Harper](https://engineering.fb.com/author/inna-harper/ "Posts by Inna Harper"), [Ke Mao](https://engineering.fb.com/author/ke-mao/ "Posts by Ke Mao"), [Jillian Ritchey](https://engineering.fb.com/author/jillian-ritchey/ "Posts by Jillian Ritchey"), [Hervé Robert](https://engineering.fb.com/author/herve-robert/ "Posts by Hervé Robert"), [Shubho Sengupta](https://engineering.fb.com/author/shubho-sengupta/ "Posts by Shubho Sengupta")

## WHAT IT IS

[Meta’s Automated Compliance Hardening (ACH) tool](https://arxiv.org/pdf/2501.12862) is a system for mutation-guided, LLM-based test generation. ACH hardens platforms against regressions by generating undetected faults (mutants) in source code that are specific to a given area of concern and using those same mutants to generate tests. When applied to privacy, for example, ACH automates the process of searching for privacy-related faults and preventing them from entering our systems in the future, ultimately hardening our code bases to reduce risk of any privacy regression.

ACH automatically generates unit tests that target a particular kind of fault. We describe the faults we care about to ACH in plain text. The description can be incomplete, and even self-contradictory, yet ACH still generates tests that it proves will catch bugs of the kind described.

Traditionally, automated test generation techniques sought merely to increase code coverage. As every tester knows, this is only part of the solution because increasing coverage doesn’t necessarily find faults.   
  
ACH is a radical departure from this tradition, because it targets specific faults, rather than uncovered code, although it often also increases coverage in the process of targeting faults. Furthermore, because ACH is founded on the principles of [Assured LLM-based Software Engineering](https://arxiv.org/abs/2402.04380), it keeps verifiable assurances that its tests do catch the kind of faults described.

Our new research paper, “[Mutation-Guided LLM-based Test Generation at Meta](https://arxiv.org/pdf/2501.12862),” gives details of the underlying scientific foundations for ACH and how we apply ACH to privacy testing, but this approach can be applied to any sort of regression testing.

## HOW IT WORKS

Mutation testing, where faults (mutants) are deliberately introduced into source code (using version control to keep them away from production) to assess how well an existing testing framework can detect these changes, has been [researched for decades](https://web.eecs.umich.edu/~weimerw/2022-481F/readings/mutation-testing.pdf). But, despite this, mutation testing has remained difficult to deploy. 

[In earlier approaches](http://crest.cs.ucl.ac.uk/fileadmin/crest/sebasepaper/JiaH10.pdf), mutants themselves would be automatically generated (most often using a rule-based approach). But this method would result in mutants that weren’t particularly realistic in terms of how much of a concern they actually represent.

On top of that, even with the mutants being automatically generated, humans would still have to manually write the tests that would kill the mutants (catch the faults).

Writing these tests is a painstaking and laborious process. So engineers were faced with a two-pronged issue: Even after doing all of the work to write a test to catch a mutant, there was no guarantee the test would even catch the automatically-generated mutant. 

By leveraging LLMs, we can generate mutants that represent realistic concerns and also save on human labor by generating tests to catch the faults automatically as well. ACH marries automated test generation techniques with the capabilities of large language models (LLMs) to generate mutants that are highly relevant to an area of testing concern as well as tests that are guaranteed to catch bugs that really matter.

Broadly, ACH works in three steps:

1. An engineer describes the kind of bugs they’re concerned about.
2. ACH uses that description to automatically generate lots of bugs.
3. ACH uses the generated bugs to automatically generate lots of tests that catch them.

At Meta we’ve [applied ACH-assisted testing to several of our platforms](https://arxiv.org/pdf/2501.12862), including Facebook Feed, Instagram, Messenger, and WhatsApp. Based on our own testing, we’ve concluded that engineers found ACH useful for hardening code against specific concerns and found other benefits even when tests generated by ACH don’t directly tackle a specific concern.

![](https://engineering.fb.com/wp-content/uploads/2025/02/Meta-ACH-system-chart.png?w=1024)

A top-level overview of the architecture of the ACH system. The system leverages LLMs to generate faults, check them against possible equivalents, and then generate tests to catch those faults.

## WHY IT MATTERS

Meta has a very large number of data systems and uses [many different programming languages](https://engineering.fb.com/2022/07/27/developer-tools/programming-languages-endorsed-for-server-side-use-at-meta/), frameworks, and services to power our family of apps and products. But, how are our thousands of engineers across the world ensuring that their code is reliable and won’t generate bugs that would negatively impact application performance, leading to privacy risk? The answer lies with LLMs. 

LLM-based test generation and LLM-based mutant generation are not new, but this is the first time they’ve been combined and deployed in large-scaled industrial systems. Generating mutants and the tests to kill them have been traditionally difficult processes to scale. Since LLMs are probabilistic and don’t need to rely on rigidly defined rules to make decisions, they allow us to tackle both sides of this equation – generating mutations and tests to kill them – very efficiently and with a high level of accuracy. 

This new approach significantly modernizes this form of automated test generation and helps software engineers take in concerns from a variety of sources (previous faults, colleagues, user requirements, regulatory requirements, etc.) and efficiently convert them from freeform text into actionable tests – with the guarantee that the test will catch the fault they’re looking for.

ACH can be applied to any class of faults and have a significant impact on hardening against future regressions and optimizing testing itself.

## WHAT’S NEXT

Our novel approach combines LLM-based test generation and mutant generation to help automate complex technical organizational workflows in this space. This innovation has the potential to simplify risk assessments, reduce cognitive load for developers, and ultimately create a safer online ecosystem. We’re committed to expanding deployment areas, developing methods to measure mutant relevance, and detecting existing faults to drive industry-wide adoption of automated test generation in compliance.

We will be sharing more developments and encourage you to watch this space.

## READ THE PAPER

[Mutation-Guided LLM-based Test Generation at Meta](https://arxiv.org/pdf/2501.12862)

### Share this:

* [Share on Facebook (Opens in new window)
  Facebook](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=facebook)
* [Share on Threads (Opens in new window)
  Threads](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=threads)
* [Share on WhatsApp (Opens in new window)
  WhatsApp](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=jetpack-whatsapp)
* [Share on LinkedIn (Opens in new window)
  LinkedIn](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=linkedin)
* [Share on Reddit (Opens in new window)
  Reddit](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=reddit)
* [Share on X (Opens in new window)
  X](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=x)
* [Share on Bluesky (Opens in new window)
  Bluesky](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=bluesky)
* [Share on Mastodon (Opens in new window)
  Mastodon](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=mastodon)
* [Share on Hacker News (Opens in new window)
  Hacker News](https://engineering.fb.com/2025/02/05/security/revolutionizing-software-testing-llm-powered-bug-catchers-meta-ach/?share=custom-1699562127)
* [Email a link to a friend (Opens in new window)
  Email](mailto:?subject=%5BShared%20Post%5D%20Revolutionizing%20software%20testing%3A%20Introducing%20LLM-powered%20bug%20catchers&body=https%3A%2F%2Fengineering.fb.com%2F2025%2F02%2F05%2Fsecurity%2Frevolutionizing-software-testing-llm-powered-bug-catchers-meta-ach%2F&share=email)

### Read More in ML Applications

[View All](https://engineering.fb.com/category/ml-applications/)

![](https://engineering.fb.com/wp-content/uploads/2026/06/Privacy-Aware-Infrastructure-in-the-AI-Native-Era-HERO.png?w=580&h=326&crop=1)

JUN 25, 2026

[Privacy-Aware Infrastructure in the AI-Native Era: An Asset Classification Case Study](https://engineering.fb.com/2026/06/25/security/privacy-aware-infrastructure-in-the-ai-native-era-an-asset-classification-case-study/)

![](https://engineering.fb.com/wp-content/uploads/2026/05/SilverTorch-Hero-Final.png?w=580&h=326&crop=1)

MAY 26, 2026

[SilverTorch: Index as Model — A New Retrieval Paradigm for Recommendation Systems](https://engineering.fb.com/2026/05/26/ml-applications/silvertorch-index-as-model-new-retrieval-paradigm-recommendation-systems/)

![](https://engineering.fb.com/wp-content/uploads/2026/05/Meta-Tech-Podcast-episode-85-Facebook-Friend-Bubbles.webp?w=580&h=326&crop=1)

MAY 13, 2026

[Reel Friends: Building Social Discovery that Scales to Billions](https://engineering.fb.com/2026/05/13/ml-applications/reel-friends-building-social-discovery-that-scales-to-billions/)

![](https://engineering.fb.com/wp-content/uploads/2026/04/Modernizing-FB-Groups-search-Hero-2.png?w=580&h=326&crop=1)

APR 21, 2026

[Modernizing the Facebook Groups Search to Unlock the Power of Community Knowledge](https://engineering.fb.com/2026/04/21/ml-applications/modernizing-the-facebook-groups-search-to-unlock-the-power-of-community-knowledge/)

![](https://engineering.fb.com/wp-content/uploads/2026/04/capacity_efficiency_hero_white_option_5_1775676974.png?w=580&h=326&crop=1 "capacity_efficiency_hero_white_option_5_1775676974")

APR 16, 2026

[Capacity Efficiency at Meta: How Unified AI Agents Optimize Performance at Hyperscale](https://engineering.fb.com/2026/04/16/developer-tools/capacity-efficiency-at-meta-how-unified-ai-agents-optimize-performance-at-hyperscale/)

![](https://engineering.fb.com/wp-content/uploads/2026/04/Compass-Not-Enclycopedia-Hero.png?w=580&h=326&crop=1)

APR 6, 2026

[How Meta Used AI to Map Tribal Knowledge in Large-Scale Data Pipelines](https://engineering.fb.com/2026/04/06/developer-tools/how-meta-used-ai-to-map-tribal-knowledge-in-large-scale-data-pipelines/)

### Related Posts

---

[![](https://engineering.fb.com/wp-content/uploads/2022/04/Eng-Blog-Self-Serve-Hero-Images-DEBUGGING-203-Blue.jpg?w=580&h=326&crop=1)

Jun 24, 2024

#### Leveraging AI for efficient incident response](https://engineering.fb.com/2024/06/24/data-infrastructure/leveraging-ai-for-efficient-incident-response/)

[![](https://engineering.fb.com/wp-content/uploads/2024/03/Logarithm-hero.png?w=580&h=326&crop=1)

Mar 18, 2024

#### Logarithm: A logging engine for AI training workflows and services](https://engineering.fb.com/2024/03/18/data-infrastructure/logarithm-logging-engine-ai-training-workflows-services-meta/)

[![](https://engineering.fb.com/wp-content/uploads/2023/12/HawkEye-Hero-2B.png?w=580&h=326&crop=1)

Dec 19, 2023

#### AI debugging at Meta with HawkEye](https://engineering.fb.com/2023/12/19/data-infrastructure/hawkeye-ai-debugging-meta/)

### Related Positions

---

* [Computer Vision Engineer, Reality Labs

  REDMOND, US](https://www.metacareers.com/jobs/1533572801746755/)
* [Software Engineer (Technical Leadership)

  SUNNYVALE, US](https://www.metacareers.com/jobs/2282367178830920/)
* [Software Engineer (Technical Leadership)

  BELLEVUE, US](https://www.metacareers.com/jobs/2282367178830920/)
* [Software Engineer (Technical Leadership)

  MENLO PARK, US](https://www.metacareers.com/jobs/2282367178830920/)
* [Software Engineer (Technical Leadership)

  NEW YORK, US](https://www.metacareers.com/jobs/2282367178830920/)

[See All Jobs](https://www.metacareers.com)

### Available Positions

---

* [Computer Vision Engineer, Reality Labs

  REDMOND, US](https://www.metacareers.com/jobs/1533572801746755/)
* [Software Engineer (Technical Leadership)

  SUNNYVALE, US](https://www.metacareers.com/jobs/2282367178830920/)
* [Software Engineer (Technical Leadership)

  BELLEVUE, US](https://www.metacareers.com/jobs/2282367178830920/)
* [Software Engineer (Technical Leadership)

  MENLO PARK, US](https://www.metacareers.com/jobs/2282367178830920/)
* [Software Engineer (Technical Leadership)

  NEW YORK, US](https://www.metacareers.com/jobs/2282367178830920/)

[See All Jobs](https://www.metacareers.com)

### Technology at Meta

* ![footer-fb-engineering](/wp-content/themes/code-fb-com/img/meta_logo.png)

  Engineering at Meta - X

  Follow
* ![footer-AI](/wp-content/themes/code-fb-com/img/meta_logo.png)

  AI at Meta

  [Read](https://ai.meta.com/blog/)
* ![footer-developers](/wp-content/themes/code-fb-com/img/meta_logo.png)

  Meta Quest Blog

  [Read](https://www.meta.com/blog/quest/)
* ![footer-developers](/wp-content/themes/code-fb-com/img/meta_logo.png)

  Meta for Developers

  [Read](https://developers.facebook.com/)
* ![footer-bug-bounty](/wp-content/themes/code-fb-com/img/meta_logo.png)

  Meta Bug Bounty

  [Learn more](https://bugbounty.meta.com/)
* ![footer-rss](/wp-content/themes/code-fb-com/img/rss.png)

  RSS

  [Subscribe](https://code.facebook.com/posts/rss/)

### Open Source

Meta believes in building community through open source technology. Explore our latest projects in Artificial Intelligence, Data Infrastructure, Development Tools, Front End, Languages, Platforms, Security, Virtual Reality, and more.

* ![android](/wp-content/themes/code-fb-com/img/android.png)

  ANDROID
* ![ios](/wp-content/themes/code-fb-com/img/ios.png)

  iOS
* ![web](/wp-content/themes/code-fb-com/img/web.png)

  WEB
* ![backend](/wp-content/themes/code-fb-com/img/backend.png)

  BACKEND
* ![hardware](/wp-content/themes/code-fb-com/img/hardware.png)

  HARDWARE

Learn More

[![Meta](https://engineering.fb.com/wp-content/themes/code-fb-com/img/meta_logo_full.svg)](https://about.facebook.com/)

Engineering at Meta is a technical news resource for engineers interested in how we solve large-scale technical challenges at Meta.

* [Home](https://engineering.fb.com)
* [Company Info](https://about.meta.com/)
* [Careers](https://www.metacareers.com/?ref=engineering.fb.com)

© 2026 Meta

* [Terms](https://www.facebook.com/policies)
* [Privacy](https://www.facebook.com/privacy/policy)
* [Cookies](/privacy)
* [Help](https://www.facebook.com/help)

To help personalize content, tailor and measure ads and provide a safer experience, we use cookies. By clicking or navigating the site, you agree to allow our collection of information on and off Facebook through cookies. Learn more, including about available controls: [Cookie Policy](/privacy)

Accept
