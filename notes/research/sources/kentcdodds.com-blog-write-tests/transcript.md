---
url: https://kentcdodds.com/blog/write-tests
title: Write tests. Not too many. Mostly integration.
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

33,426 [what's this?](/teams#read-rankings)

[Login](/login)

* 0.048
* 0
* 0

## Write tests. Not too many. Mostly integration.

July 13th, 2019 — 6 min read

[Login to favorite](/login "Login to favorite")

![by Elena Cordery](data:image/webp;base64,UklGRpoDAABXRUJQVlA4WAoAAAAgAAAAYwAAlQAASUNDUDACAAAAAAIwQURCRQIQAABtbnRyUkdCIFhZWiAHzwAGAAMAAAAAAABhY3NwQVBQTAAAAABub25lAAAAAAAAAAAAAAAAAAAAAQAA9tYAAQAAAADTLUFEQkUAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAApjcHJ0AAAA/AAAADJkZXNjAAABMAAAAGt3dHB0AAABnAAAABRia3B0AAABsAAAABRyVFJDAAABxAAAAA5nVFJDAAAB1AAAAA5iVFJDAAAB5AAAAA5yWFlaAAAB9AAAABRnWFlaAAACCAAAABRiWFlaAAACHAAAABR0ZXh0AAAAAENvcHlyaWdodCAxOTk5IEFkb2JlIFN5c3RlbXMgSW5jb3Jwb3JhdGVkAAAAZGVzYwAAAAAAAAARQWRvYmUgUkdCICgxOTk4KQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWFlaIAAAAAAAAPNRAAEAAAABFsxYWVogAAAAAAAAAAAAAAAAAAAAAGN1cnYAAAAAAAAAAQIzAABjdXJ2AAAAAAAAAAECMwAAY3VydgAAAAAAAAABAjMAAFhZWiAAAAAAAACcGAAAT6UAAAT8WFlaIAAAAAAAADSNAACgLAAAD5VYWVogAAAAAAAAJjEAABAvAAC+nFZQOCBEAQAAEA8AnQEqZACWAD6dSJ1LKCupoab5mlFwE4lpANMsqLb0xOGDKMqN0BSVWO8tvu+IpnUpeccFXv/lwK2YPV+9qUhk2oe0dzpWG0nvl4d16+lxtf03h7ErrARgZWyJpv1PkakRKNrGHFZv1v/tK3R4n6d9muDdPpsRgYhbazlU0TywAP7wMtH5d4JpsgKmyDpU0KpnxUm+Dg2arjHqgsR/SLp/mvXYA7xFxSkJE6sS8mP6CbJWAmgcMxwJ/4FnPYupJRVLQqC60xvBoYDSn3vmOCWmsiQdTxRigcE08MPJBBSXkYNWNDtZqtYl+5EAj5Iciq5S62c6OJ5PzXqeaVNNQsRr8FzY/ttbr4bWiL8XXm252r6iRAxMk8ryE+0u9mLIrGFVCRN49Oyg1aTJsjBP0kaKYjiicNIBxbPNQglJkg+6QAAA)![by Elena Cordery](https://res.cloudinary.com/kentcdodds-com/image/upload/w_1517,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1469598614039-ccfeb0a21111 "Photo by Elena Cordery")

* [Portuguese](https://medium.com/@sergioamjr91/escreva-testes-não-muitos-mas-mais-de-integração-7ebebf225516)
* [日本語](https://makotot.dev/posts/write-tests-translation-ja)

[Add translation](https://github.com/kentcdodds/kentcdodds.com/blob/main/CONTRIBUTING.md#translation-contributions)

I've given this blog post as a talk which you can watch here:

A while back, [Guillermo Rauch‏](https://x.com/rauchg) (creator of
[Socket.io](https://socket.io) and founder of [Zeit.co](https://zeit.co) (the
company behind a ton of the awesome stuff coming out lately))
[tweeted something profound](https://x.com/rauchg/status/807626710350839808):

[![Guillermo Rauch avatar](https://pbs.twimg.com/profile_images/1783856060249595904/8TfcCN0r_bigger.jpg)

Guillermo Rauch
@rauchg](https://x.com/rauchg)
> Write tests. Not too many. Mostly integration.

[4:43 PM (UTC) · December 10th, 2016](https://x.com/rauchg/status/807626710350839808)

[26](https://x.com/rauchg/status/807626710350839808)

[1,408](https://x.com/intent/like?tweet_id=807626710350839808)

> ***Write tests. Not too many. Mostly integration.***

This is deep, albeit short, so let's dive in:

## [Write tests.](#write-tests)

Yes, for most projects you should write automated tests. You should if you value
your time anyway. Much better to catch a bug locally from the tests than getting
a call at 2:00 in the morning and fix it then. **Often I find myself saving time
when I put time in to write tests.** It may or may not take longer to implement
what I'm building, but I (and others) will almost definitely save time
maintaining it.

The thing you should be thinking about when writing tests is how much confidence
they bring you that your project is free of bugs. Static typing and linting
tools like [TypeScript](https://www.typescriptlang.org) and
[ESLint](https://eslint.org) can get you a remarkable amount of confidence, and
if you're not using these tools I highly suggest you give them a look. That
said, **even a strongly typed language should have tests.** Typing and linting
can't ensure your business logic is free of bugs. So you can still seriously
increase your confidence with a good test suite.

## [Not too many.](#not-too-many)

I've heard managers and teams mandating 100% code coverage for applications.
That's a really bad idea. The problem is that **you get diminishing returns on
your tests as the coverage increases much beyond 70%** (I made that number up...
no science there). Why is that? Well, when you strive for 100% all the time, you
find yourself spending time testing things that really don't need to be tested.
Things that really have no logic in them at all (so any bugs could be caught by
ESLint and Flow). *Maintaining tests like this actually really slow you and your
team down.*

You may also find yourself testing implementation details just so you can make
sure you get that one line of code that's hard to reproduce in a test
environment. You *really* want to avoid testing implementation details because
it doesn't give you very much confidence that your application is working and it
slows you down when refactoring. **You should very rarely have to change tests
when you refactor code.**

I should mention that almost all of my open source projects have 100% code
coverage. This is because most of my open source projects are smaller libraries
and tools that are reusable in many different situations (a breakage could lead
to a serious problem in a lot of consuming projects) and they're relatively easy
to get 100% code coverage on anyway.

## [Mostly integration.](#mostly-integration)

There are all sorts of different types of testing (check out my 5 minute talk
about it at Fluent Conf:
["What we can learn about testing from the wheel"](https://youtu.be/Da9wfQ0frGA?list=PLV5CVI1eNcJgNqzNwcs4UKrlJdhfDjshf)).
They each have trade-offs. The three most common forms of testing we're talking
about when we talk of automated testing are: Unit, Integration, and End to End.

Here's [a slide](http://slides.com/kentcdodds/testing-workshop#/4/8) from my
Frontend Masters workshop:
["Testing JavaScript Applications"](https://frontendmasters.com/courses/testing-javascript).

![testing pyramid](https://res.cloudinary.com/kentcdodds-com/image/upload/f_auto,q_auto,dpr_2.0,w_1600/v1625033547/kentcdodds.com/content/blog/write-tests/2.png)

This testing pyramid is a combination of one I got from
[Martin Fowler's blog](https://martinfowler.com/bliki/TestPyramid.html) and one
I got from
[the Google Testing blog](https://testing.googleblog.com/2015/04/just-say-no-to-more-end-to-end-tests.html).

As indicated here, the pyramid shows from bottom to top: Unit, Integration, E2E.
As you move up the pyramid the tests get slower to write/run and more expensive
(in terms of time and resources) to run/maintain. It's meant to indicate that
you should spend more of your time on unit tests due to these factors.

One thing that it doesn't show though is that **as you move up the pyramid, the
confidence quotient of each form of testing increases.** You get more bang for
your buck. So while E2E tests may be slower and more expensive than unit tests,
they bring you much more confidence that your application is working as
intended.

[![Kent C. Dodds 🏹 avatar](https://pbs.twimg.com/profile_images/1567269493608714241/6ACZo99k_bigger.jpg)

Kent C. Dodds 🏹
@kentcdodds](https://x.com/kentcdodds)
> The TestPyramid blog post by [@martinfowler](https://x.com/martinfowler) ([bliki: Test Pyramid](https://martinfowler.com/bliki/TestPyramid.html)) finishes with this note.
> Our tools are awesome now and this assumption is less true. This is why I say goodby to the pyramid 🔼 and hello to the trophy 🏆 [Static vs Unit vs Integration vs E2E Testing for Frontend Apps](https://kentcdodds.com/blog/static-vs-unit-vs-integration-vs-e2e-tests)

[![Tweet media](https://pbs.twimg.com/media/D9bxhvGU0AAheLo.jpg)](https://x.com/kentcdodds/status/1141365123296051201)

[3:20 PM (UTC) · June 19th, 2019](https://x.com/kentcdodds/status/1141365123296051201)

[2](https://x.com/kentcdodds/status/1141365123296051201)

[57](https://x.com/intent/like?tweet_id=1141365123296051201)

As noted, our tools have moved beyond the assumption in Martin's original
Testing Pyramid concept. This is why I created "The Testing Trophy" 🏆

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

It doesn't matter if your component `<A />` renders component `<B />` with props
`c` and `d` if component `<B />` actually breaks if prop `e` is not supplied. So
while having some unit tests to verify these pieces work in isolation isn't a
bad thing, *it doesn't do you any good if you don't **also** verify that they
work together properly.* And you'll find that by testing that they work together
properly, you often don't need to bother testing them in isolation.

**Integration tests strike a great balance on the trade-offs between confidence
and speed/expense.** This is why it's advisable to spend *most* (not all, mind
you) of your effort there.

For more on this read [Testing Implementation
Details](/blog/testing-implementation-details). For more about the different
distinctions of tests, read [Static vs Unit vs Integration vs E2E Testing for
Frontend Apps](/blog/static-vs-unit-vs-integration-vs-e2e-tests)

## [How to write more integration tests](#how-to-write-more-integration-tests)

The line between integration and unit tests is a little bit fuzzy. Regardless, I
think the biggest thing you can do to write more integration tests is to **stop
mocking so much stuff**. *When you mock something you're removing all confidence
in the integration between what you're testing and what's being mocked.* I
understand that [sometimes it can't be helped](/blog/the-merits-of-mocking)
(though [some would disagree](https://youtu.be/EaxDl5NPuCA)). You don't
*actually* want to send emails or charge credit cards every test, but most of
the time you can avoid mocking and you'll be better for it.

**If you're doing React, then this includes shallow rendering.** For more on
this, read
[Why I Never Use Shallow Rendering](/blog/why-i-never-use-shallow-rendering).

## [Conclusion](#conclusion)

I don't think that anyone can argue that testing software is a waste of time.
The biggest challenge is [knowing what to test](/blog/how-to-know-what-to-test)
and how to test it in a way that gives
[true confidence](/blog/confidently-shipping-code) rather than the false
confidence of
[testing implementation details](/blog/testing-implementation-details).

I hope this is helpful to you and I wish you the best luck in your goals to find
confidence in shipping your applications!

Testing course

![Illustration of a trophy](https://res.cloudinary.com/kentcdodds-com/image/upload/w_537,q_auto,f_auto/v1746462314/kentcdodds.com/pages/courses/v2/trophy)

## Testing JavaScript

Ship Apps with Confidence

[Visit course](https://testingjavascript.com)

33,426 [what's this?](/teams#read-rankings)

* 0.048
* 0
* 0

[Login](/login)

[Login to favorite](/login "Login to favorite")

[Post this article](https://x.com/intent/tweet?url=https%3A%2F%2Fkentcdodds.com%2Fblog%2Fwrite-tests&text=I+just+read+%22Write+tests.+Not+too+many.+Mostly+integration.%22+by+%40kentcdodds%0A%0A)

[Discuss on 𝕏](https://x.com/search?q=https%3A%2F%2Fkentcdodds.com%2Fblog%2Fwrite-tests)•[Edit on GitHub](https://github.com/kentcdodds/kentcdodds.com/edit/main/services/site/content/blog/write-tests.mdx)

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

[![Photo by [Robert Collins](https://unsplash.com/photos/tvc5imO5pXk)](https://res.cloudinary.com/kentcdodds-com/image/upload/c_fill,w_955,ar_3:4,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1502086223501-7ea6ecd79368 "How I learn an Open Source Codebase")

May 14th, 2018 — 4 min read

How I learn an Open Source Codebase](/blog/how-i-learn-an-open-source-codebase)Click to copy url

[![Photo by [Alexandru Goman](https://unsplash.com/photos/CM-qccHaQ04)](https://res.cloudinary.com/kentcdodds-com/image/upload/c_fill,w_955,ar_3:4,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1522424427542-e6fc86ff5253 "AHA Testing 💡")

April 7th, 2019 — 10 min read

AHA Testing 💡](/blog/aha-testing)Click to copy url

[![brown and white cat in shallow focus shot](https://res.cloudinary.com/kentcdodds-com/image/upload/c_fill,w_955,ar_3:4,q_auto,f_auto,b_rgb:e6e9ee/unsplash/photo-1525785967371-87ba44b3e6cf "Get a catch block error message with TypeScript")

October 28th, 2021 — 5 min read

Get a catch block error message with TypeScript](/blog/get-a-catch-block-error-message-with-typescript)Click to copy url

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
