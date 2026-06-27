---
url: https://kentcdodds.com/blog/the-testing-trophy-and-testing-classifications
title: The Testing Trophy and Testing Classifications
fetched: 2026-06-27
raw: raw.html
transport: curl
capture_status: ok
---

[# Kent C. Dodds](/)

* [Blog](/blog)
* [Talks](/talks)
* [Courses](/courses)
* [Better](/better)
* [Discord](/discord)
* [Calls](/calls)
* [About](/about)

[Home](/)[Blog](/blog)[Talks](/talks)[Courses](/courses)[Better](/better)[Discord](/discord)[Calls](/calls)[About](/about)

[![Kody Profile in Gray](https://res.cloudinary.com/kentcdodds-com/image/upload/c_pad,w_80,h_80,q_auto,f_auto/kentcdodds.com/illustrations/kody/kody_profile_gray)](/login)

[Back to overview](/blog)

12,537 [what's this?](/teams#read-rankings)

[Login](/login)

* 0.05
* 0
* 0

## The Testing Trophy and Testing Classifications

June 3rd, 2021 — 7 min read

[Login to favorite](/login "Login to favorite")

![by Fauzan Saari](data:image/webp;base64,UklGRuwAAABXRUJQVlA4IOAAAADQBwCdASpkADgAPrlSn0wppKaoKl46iTAXCWUAxzAdEQrE4+l/MC4CsWyANsW1z8M512Po1w09qSXQZ59q6+4NmL21UJ1eQAD+7A7WOo5Ig3uWiQdGJqh+trsjFwY3KMtR8OWRY9WfjnmRfuwX7zHAbEd7HcZUeD0FLL9ltWQUJAitqdaYdKeAHtuLk0TepYZMBR/BBZgrEz9jrKvnHbQ1Fz12sG8aQafdgSHbffn66Ddldx1X8wPkZwhtmDRymFPlLLhEy9fYck/p/JeyR9jvrOokZmvGmOYJgG0sBAAAAA==)![by Fauzan Saari](https://res.cloudinary.com/kentcdodds-com/image/upload/w_1517,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1527871454777-032ec3f75edc "Photo by Fauzan Saari")

No translations available.[Add translation](https://github.com/kentcdodds/kentcdodds.com/blob/main/CONTRIBUTING.md#translation-contributions)

Allow me to indulge in a little personal history. If you're unfamiliar with the
testing trophy, here it is:

![Illustration of a trophy separated into 4 sections labeled from top to
bottom: End to End, Integration, Unit,
Static](https://res.cloudinary.com/kentcdodds-com/image/upload/f_auto,q_auto,dpr_2.0,w_1600/v1622744540/kentcdodds.com/blog/the-testing-trophy-and-testing-classifications/trophy_wx9aen.png)

I initially introduced this in a tweet with a quick drawing I made with Google
Drive:

[![Kent C. Dodds 🏹 avatar](https://pbs.twimg.com/profile_images/1567269493608714241/6ACZo99k_bigger.jpg)

Kent C. Dodds 🏹
@kentcdodds](https://x.com/kentcdodds)
> "The Testing Trophy" 🏆
> A general guide for the \*\*return on investment\*\* 🤑 of the different forms of testing with regards to testing JavaScript applications.
> - End to end w/ [@Cypress\_io](https://x.com/Cypress_io) ⚫️
> - Integration & Unit w/ [@fbjest](https://x.com/fbjest) 🃏
> - Static w/ [@flowtype](https://x.com/flowtype) 𝙁 and [@geteslint](https://x.com/geteslint) ⬣

[![Tweet media](https://pbs.twimg.com/media/DVUoM94VQAAzuws.jpg)](https://x.com/kentcdodds/status/960723172591992832)

[3:53 AM (UTC) · February 6th, 2018](https://x.com/kentcdodds/status/960723172591992832)

[20](https://x.com/kentcdodds/status/960723172591992832)

[739](https://x.com/intent/like?tweet_id=960723172591992832)

I came up with this idea after publishing a blog post titled
["Write tests. Not too many. Mostly integration."](/blog/write-tests):

[![Kent C. Dodds 🏹 avatar](https://pbs.twimg.com/profile_images/1567269493608714241/6ACZo99k_bigger.jpg)

Kent C. Dodds 🏹
@kentcdodds](https://x.com/kentcdodds)
> I just published “Write tests. Not too many. Mostly integration.” [Write tests. Not too many. Mostly integration.](https://kentcdodds.com/blog/write-tests) 🕶

[![Referenced media](https://res.cloudinary.com/kentcdodds-com/image/upload/$th_1256,$tw_2400,$gw_$tw_div_24,$gh_$th_div_12/co_rgb:a9adc1,c_fit,g_north_west,w_$gw_mul_14,h_$gh,x_$gw_mul_1.5,y_$gh_mul_1.3,l_text:kentcdodds.com:Matter-Regular.woff2_50:Check%2520out%2520this%2520article/co_white,c_fit,g_north_west,w_$gw_mul_13.5,h_$gh_mul_7,x_$gw_mul_1.5,y_$gh_mul_2.3,l_text:kentcdodds.com:Matter-Regular.woff2_110:Write%2520tests.%2520Not%2520too%2520many.%2520Mostly%2520integration./c_fit,g_north_west,r_max,w_$gw_mul_4,h_$gh_mul_3,x_$gw,y_$gh_mul_8,l_kent:profile-transparent/co_rgb:a9adc1,c_fit,g_north_west,w_$gw_mul_5.5,h_$gh_mul_4,x_$gw_mul_4.5,y_$gh_mul_9,l_text:kentcdodds.com:Matter-Regular.woff2_70:Kent%20C.%20Dodds/co_rgb:a9adc1,c_fit,g_north_west,w_$gw_mul_9,x_$gw_mul_4.5,y_$gh_mul_9.8,l_text:kentcdodds.com:Matter-Regular.woff2_40:kentcdodds.com%252Fblog%252Fwrite-tests/c_fill,ar_3:4,r_12,g_east,h_$gh_mul_10,x_$gw,l_unsplash:photo-1469598614039-ccfeb0a21111/c_fill,w_$tw,h_$th/kentcdodds.com/social-background.png)

Write tests. Not too many. Mostly integration.

[Guillermo Rauch](https://x.com/rauchg) [tweeted](https://x.com/rauchg/status/807626710350839808) this a while back. Let's take a dive into what it means.

kentcdodds.com](https://kentcdodds.com/blog/write-tests)

[2:22 PM (UTC) · October 16th, 2017](https://x.com/kentcdodds/status/919931474828181505)

[16](https://x.com/kentcdodds/status/919931474828181505)

[360](https://x.com/intent/like?tweet_id=919931474828181505)

Which was my take on [Guillermo Rauch's](https://x.com/rauchg) tweet from
about a year earlier:

[![Guillermo Rauch avatar](https://pbs.twimg.com/profile_images/1783856060249595904/8TfcCN0r_bigger.jpg)

Guillermo Rauch
@rauchg](https://x.com/rauchg)
> Write tests. Not too many. Mostly integration.

[4:43 PM (UTC) · December 10th, 2016](https://x.com/rauchg/status/807626710350839808)

[26](https://x.com/rauchg/status/807626710350839808)

[1,409](https://x.com/intent/like?tweet_id=807626710350839808)

I can't speak for Guillermo, but I agreed so strongly with what he said because
of my experience as a UI engineer and how I personally had come to understand
the term "integration" in this context.

Especially at that time in my career, almost all the code I wrote either ran
directly in a browser or was intended for a tool that would help me run code in
a browser. So for me naturally the terms "unit", "integration", and "end-to-end"
would be viewed through the lens of that experience. In fact, I added "static"
to the trophy because in the world of JavaScript that's not a given like it is
in the predominant languages when
[the testing pyramid](https://martinfowler.com/bliki/TestPyramid.html) was
introduced.

The reason I explain this background is to help you understand the way the
Testing Trophy is intended to be interpreted. I never considered whether it
applied to microservices or even backend services at all. I considered my
codebase in isolation and attempted to categorize the types of tests I could
write within the confines of my own code ownership. I always thought of
end-to-end tests as the place where you attempt to validate that things work
without any (or more practically "as little as possible") mocking in place.

So that left me with categorizing tests on my own code into either "unit" or
"integration". I consider a "unit" to be a single function, class, or object
that contains logic. So here's how I decided to (loosely) categorize them:

* Unit tests are those which test units which either have no dependencies
  (collaborators) or which have those mocked for the test.
* Integration tests are those which test multiple units integrating with one
  another.

Eventually, I created [Testing Library](https://testing-library.com) to
encourage the kinds of testing practices that worked best for me:

[![Kent C. Dodds 🏹 avatar](https://pbs.twimg.com/profile_images/1567269493608714241/6ACZo99k_bigger.jpg)

Kent C. Dodds 🏹
@kentcdodds](https://x.com/kentcdodds)
> I just published “Introducing the react-testing-library 🐐” [Introducing the react-testing-library 🐐](https://kentcdodds.com/blog/introducing-the-react-testing-library)

[![Referenced media](https://res.cloudinary.com/kentcdodds-com/image/upload/$th_1256,$tw_2400,$gw_$tw_div_24,$gh_$th_div_12/co_rgb:a9adc1,c_fit,g_north_west,w_$gw_mul_14,h_$gh,x_$gw_mul_1.5,y_$gh_mul_1.3,l_text:kentcdodds.com:Matter-Regular.woff2_50:Check%2520out%2520this%2520article/co_white,c_fit,g_north_west,w_$gw_mul_13.5,h_$gh_mul_7,x_$gw_mul_1.5,y_$gh_mul_2.3,l_text:kentcdodds.com:Matter-Regular.woff2_110:Introducing%2520the%2520react-testing-library/c_fit,g_north_west,r_max,w_$gw_mul_4,h_$gh_mul_3,x_$gw,y_$gh_mul_8,l_kent:profile-transparent/co_rgb:a9adc1,c_fit,g_north_west,w_$gw_mul_5.5,h_$gh_mul_4,x_$gw_mul_4.5,y_$gh_mul_9,l_text:kentcdodds.com:Matter-Regular.woff2_70:Kent%20C.%20Dodds/co_rgb:a9adc1,c_fit,g_north_west,w_$gw_mul_9,x_$gw_mul_4.5,y_$gh_mul_9.8,l_text:kentcdodds.com:Matter-Regular.woff2_40:kentcdodds.com%252Fblog%252Fintroducing-the-react-testing-library/c_fill,ar_3:4,r_12,g_east,h_$gh_mul_10,x_$gw,l_kentcdodds.com:content:blog:introducing-the-react-testing-library:banner/c_fill,w_$tw,h_$th/kentcdodds.com/social-background.png)

Introducing the react-testing-library 🐐

A simpler replacement for enzyme that encourages good testing practices.

kentcdodds.com](https://kentcdodds.com/blog/introducing-the-react-testing-library)

[1:30 PM (UTC) · April 2nd, 2018](https://x.com/kentcdodds/status/980799621080391680)

[9](https://x.com/kentcdodds/status/980799621080391680)

[234](https://x.com/intent/like?tweet_id=980799621080391680)

By my own definition, Testing Library can be used to test individual React
components (unit tests), entire pages with HTTP requests mocked
[via MSW](/blog/stop-mocking-fetch) (integration tests), the full app with very
few mocks (end-to-end tests), and even
[individual React hooks](/blog/how-to-test-custom-react-hooks) if necessary
(lower level unit tests). And Testing Library is now the most popular and de
facto standard... er... testing library for React apps and increasingly the same
is happening wherever the DOM can be found. In May 2020,
[Testing Library received the "Adopt" distinction on the ThoughtWorks Technology Radar](https://www.thoughtworks.com/radar/languages-and-frameworks/react-testing-library).

I expect some will reply to this blog post with: "Why did you have to make up
your own definitions in the first place? Just use the ones that exist." So I'll
respond before you ask: "Which of the two dozen different definitions would you
like me to have chosen for my own definition?" 😂 😭 In his post about
[test shapes](https://martinfowler.com/articles/2021-test-shapes.html),
[Martin Fowler](https://x.com/martinfowler) approximates a quote of a
"test expert" who was asked in the 1990s how they define "unit test":

> “in the first morning of my training course I cover 24 different definitions
> of unit test”.

This is a sad state of affairs, and it's been that way since the 90s
unfortunately. It is what it is. I had to choose something that made sense for
me and as an educator, I had to choose something that would make the most sense
for the people I'm teaching. Judging by the response from people who have
implemented my recommendations, my decision was a good one.

When discussing whether you can prove that testing is effective,
[Tim Bray](https://x.com/timbray) (in his article
[Testing in the Twenties](https://www.tbray.org/ongoing/When/202x/2021/05/15/Testing-in-2021)),
correctly says:

> let's not kid ourselves that our software-testing tenets constitute scientific
> knowledge.

I would say this applies to everything about testing–not just whether it's
effective (it can be). Any attempt to come to a single definition for all these
terms is a futile endeavor. I remember speaking at Assert(JS) (where I gave my
talk
[Write Tests. Not too many. Mostly Integration.](https://www.youtube.com/watch?v=Fha2bVoC8SE&list=PLV5CVI1eNcJgNqzNwcs4UKrlJdhfDjshf))
and I observed how wildly different each talk was with regards to their
recommendations on testing. But as I think about it now, I think lots of the
difference could be attributed to our definitions of the terms of testing and
less on how we strive to achieve confidence.

[Justin Searls](https://x.com/searls) (who incidentally also
[spoke at Assert(JS)](https://www.youtube.com/watch?v=Af4M8GMoxi4) that year)
said it best when he tweeted:

[![Justin Searls avatar](https://pbs.twimg.com/profile_images/2360535353/20120630-face_bigger.jpg)

Justin Searls
@searls](https://x.com/searls)
> People love debating what percentage of which type of tests to write, but it's a distraction. Nearly zero teams write expressive tests that establish clear boundaries, run quickly & reliably, and only fail for useful reasons. Focus on that instead.

[![swyx avatar](https://pbs.twimg.com/profile_images/1867875781676007424/RIF4Kt7U_bigger.jpg)

swyx
@swyx](https://x.com/swyx)
> The [@MartinFowler](https://x.com/MartinFowler) Test Pyramid has fallen out of style.
> Integration > Unit tests is the new conventional wisdom.
> In frontend, we now have the "Testing Trophy" from [@rauchg](https://x.com/rauchg) and [@kentcdodds](https://x.com/kentcdodds).
> In backend, [@theburningmonk](https://x.com/theburningmonk)'s course advocates the "Testing Honeycomb" from [@SpotifyEng](https://x.com/SpotifyEng).

[![Tweet media](https://pbs.twimg.com/media/EYCwtegU0AESioV.jpg)![Tweet media](https://pbs.twimg.com/media/EYCwv8xUYAA2X0e.jpg)](https://x.com/swyx/status/1261202288476971008)

[7:50 AM (UTC) · May 15th, 2020](https://x.com/swyx/status/1261202288476971008)

[30](https://x.com/swyx/status/1261202288476971008)

[593](https://x.com/intent/like?tweet_id=1261202288476971008)

[1:58 AM (UTC) · May 15th, 2021](https://x.com/searls/status/1393385209089990659)

[16](https://x.com/searls/status/1393385209089990659)

[593](https://x.com/intent/like?tweet_id=1393385209089990659)

Classification is important so we can have conversations about this. It's
unfortunate that you pretty much need to come to a consensus on how you define
these terms before having a productive conversation. But ultimately it really
doesn't matter. As Justin says, it's a distraction. Especially when so many
codebases are living life on the edge without an automated way to have
confidence their changes are safe to deploy.

## [Conclusion](#conclusion)

Anyway, hopefully this helps to clear things up a bit. To sum up: When trying to
apply the testing trophy to your situation, think of it within the code of an
individual codebase. It definitely has applicability in backends, but I've only
considered it for monoliths not microservices or even serverless functions (and
[I agree with Tim](https://www.tbray.org/ongoing/When/202x/2021/05/15/Testing-in-2021),
most of us should probably be writing monoliths if we can).

The testing trophy (when understood) has given me (and countless other) clarity
on where to focus testing efforts. When properly interpreted, it helps me keep
this critical principle in mind:

[![Kent C. Dodds 🏹 avatar](https://pbs.twimg.com/profile_images/1567269493608714241/6ACZo99k_bigger.jpg)

Kent C. Dodds 🏹
@kentcdodds](https://x.com/kentcdodds)
> The more your tests resemble the way your software is used, the more confidence they can give you.

[3:05 AM (UTC) · March 23rd, 2018](https://x.com/kentcdodds/status/977018512689455106)

[16](https://x.com/kentcdodds/status/977018512689455106)

[1,128](https://x.com/intent/like?tweet_id=977018512689455106)

This is the guiding principle for Testing Library and it's how I think about
every testing problem I face.

**Remember,** it's all about getting a good return on your investment where
"return" is "confidence" and "investment" is "time." If we had unlimited time,
then trying to classify things wouldn't be necessary, we'd just write tests
forever! But we don't, so I hope this helps you when trying to decide where to
put your efforts.

P.S. If you'd like more of my thoughts on testing, I have
[a lot of posts on the subject on my blog](/blog?q=test). Here are a few
specific articles I recommend you read next:

* [Confidently Shipping Code](/blog/confidently-shipping-code): Why I care about
  testing.
* [Static vs Unit vs Integration vs E2E Testing for Frontend Apps](/blog/static-vs-unit-vs-integration-vs-e2e-tests):
  What these mean, why they matter, and why they don't. ⭐️ This one has code
  examples you might find instructive if you'd like more concrete examples of
  how I think about these different classifications of tests.
* [Testing Implementation Details](/blog/testing-implementation-details):
  Testing implementation details is a recipe for disaster. Why is that? And what
  does it even mean?
* [Avoid the Test User](/blog/avoid-the-test-user): How your UI code has only
  two users, but the wrong tests can add a third.
* [Should I write a test or fix a bug](/blog/should-i-write-a-test-or-fix-a-bug):
  How to prioritize tests relative to everything else.
* [How to know what to test](/blog/how-to-know-what-to-test): Practical advice
  to help you determine what to test.

React course

![Illustration of a Rocket](https://res.cloudinary.com/kentcdodds-com/image/upload/w_537,q_auto,f_auto/v1746462314/kentcdodds.com/pages/courses/v2/rocket)

## Epic React

Get Really Good at React

[Visit course](https://epicreact.dev)

Testing course

![Illustration of a trophy](https://res.cloudinary.com/kentcdodds-com/image/upload/w_537,q_auto,f_auto/v1746462314/kentcdodds.com/pages/courses/v2/trophy)

## Testing JavaScript

Ship Apps with Confidence

[Visit course](https://testingjavascript.com)

12,537 [what's this?](/teams#read-rankings)

* 0.05
* 0
* 0

[Login](/login)

[Login to favorite](/login "Login to favorite")

[Post this article](https://x.com/intent/tweet?url=https%3A%2F%2Fkentcdodds.com%2Fblog%2Fthe-testing-trophy-and-testing-classifications&text=I+just+read+%22The+Testing+Trophy+and+Testing+Classifications%22+by+%40kentcdodds%0A%0A)

[Discuss on 𝕏](https://x.com/search?q=https%3A%2F%2Fkentcdodds.com%2Fblog%2Fthe-testing-trophy-and-testing-classifications)•[Edit on GitHub](https://github.com/kentcdodds/kentcdodds.com/edit/main/services/site/content/blog/the-testing-trophy-and-testing-classifications.mdx)

![Kent C. Dodds](https://res.cloudinary.com/kentcdodds-com/image/upload/w_299,q_auto,f_auto/kent/profile-transparent)

Written by Kent C. Dodds

Kent C. Dodds is a JavaScript software engineer and teacher. Kent's taught hundreds
of thousands of people how to make the world a better place with quality software
development tools and practices. He lives with his wife and four kids in Utah.

[Learn more about Kent](/about)

#### Have a question about this article?

Bring it to the Call Kent podcast. Ask on </calls> and I may answer it on the podcast.

[Place a call](/calls)

## If you found this article helpful.

You will love these ones as well.

[![Photo by [Cytonn Photography](https://unsplash.com/photos/ZJEKICY5EXY)](https://res.cloudinary.com/kentcdodds-com/image/upload/c_fill,w_955,ar_3:4,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1521790361543-f645cf042ec4 "How to get experience as a software engineer")

August 12th, 2019 — 5 min read

How to get experience as a software engineer](/blog/how-to-get-experience-as-a-software-engineer)Click to copy url

[![Photo by [Sarah Kilian](https://unsplash.com/photos/52jRtc2S_VE)](https://res.cloudinary.com/kentcdodds-com/image/upload/c_fill,w_955,ar_3:4,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1555861496-0666c8981751 "Common mistakes with React Testing Library")

May 4th, 2020 — 15 min read

Common mistakes with React Testing Library](/blog/common-mistakes-with-react-testing-library)Click to copy url

[![Photo by [rawpixel](https://unsplash.com/photos/1Z15APktAiY)](https://res.cloudinary.com/kentcdodds-com/image/upload/c_fill,w_955,ar_3:4,q_auto,f_auto,b_rgb:e6e9ee/kentcdodds.com/content/blog/testing-implementation-details/banner "Testing Implementation Details")

August 17th, 2020 — 12 min read

Testing Implementation Details](/blog/testing-implementation-details)Click to copy url

Kent C. Dodds

Full time educator making our world better

Stay up to date

Subscribe to the newsletter to stay up to date with articles,
courses and much more!
[Learn more about the newsletter](/subscribe)

Contact

* [Email Kent](/contact)
* [Call Kent](/calls)
* [Office hours](/office-hours)

General

* [My Mission](/transparency)
* [Privacy policy](/transparency#privacy)
* [Terms of use](/transparency#terms)
* [Code of conduct](/conduct)

Sitemap

* [Home](/)
* [Blog](/blog)
* [Courses](/courses)
* [Better](/better)
* [Discord](/discord)
* [Chats Podcast](/chats)
* [Talks](/talks)
* [Testimony](/testimony)
* [Testimonials](/testimonials)
* [About](/about)
* [Resume](/resume)
* [Credits](/credits)
* [Sitemap.xml](/sitemap.xml)

Stay up to date

Subscribe to the newsletter to stay up to date with articles,
courses and much more!
[Learn more about the newsletter](/subscribe)

All rights reserved © Kent C. Dodds 2026
