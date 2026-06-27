---
url: https://martinfowler.com/bliki/TestDouble.html
title: Test Double
fetched: 2026-06-27
raw: raw.html
transport: curl
capture_status: ok
---

[![](/mf-name-white.png)](https://martinfowler.com)

* [Refactoring](https://refactoring.com)
* [Agile](/agile.html)
* [Architecture](/architecture)
* [About](/aboutMe.html)
* [Thoughtworks](https://www.thoughtworks.com/engineering)

## Topics

[Architecture](/architecture)

[Refactoring](https://refactoring.com)

[Agile](/agile.html)

[Delivery](/delivery.html)

[Microservices](/microservices)

[Data](/data)

[Testing](/testing)

[DSL](/dsl.html)

## about me

[About](/aboutMe.html)

[Books](/books)

[FAQ](/faq.html)

## content

[Videos](/videos.html)

[Content Index](/tags)

[Fragments](/fragments)

[Board Games](/boardgames)

[Photography](/photos)

## Thoughtworks

[Home](https://thoughtworks.com)

[Insights](https://thoughtworks.com/insights)

[Careers](https://thoughtworks.com/careers)

[Radar](https://thoughtworks.com/radar)

[Engineering](https://www.thoughtworks.com/engineering)

## follow

[RSS](/feed.atom)

[Mastodon](https://toot.thoughtworks.com/@mfowler)

[LinkedIn](https://www.linkedin.com/in/martin-fowler-com/)

[Bluesky](https://bsky.app/profile/martinfowler.com)

[X](https://www.twitter.com/martinfowler)

[BGG](https://boardgamegeek.com/blog/13064/martins-7th-decade)

# [Test Double](TestDouble.html)

17 January 2006

[![](/mf.jpg "Photo of Martin Fowler")](/)

[Martin Fowler](/)

[testing](/tags/testing.html)

Gerard Meszaros is [working on a book](/books/meszaros.html) to capture patterns for using
the various [Xunit](/bliki/Xunit.html) frameworks. One of the awkward things
he's run into is the various names for stubs, mocks, fakes, dummies,
and other things that people use to stub out parts of a system for
testing. To deal with this he's come up with his own vocabulary
which I think is worth spreading further.

The generic term he uses is a [Test Double](http://xunitpatterns.com/Test%20Double.html) (think stunt
double). Test Double is a generic term for any case where you
replace a production object for testing purposes. There are various
kinds of double that Gerard lists:

* **Dummy** objects are passed around but never actually
  used. Usually they are just used to fill parameter lists.
* **Fake** objects actually have working implementations, but
  usually take some shortcut which makes them not suitable for
  production (an [InMemoryTestDatabase](/bliki/InMemoryTestDatabase.html) is a good example).
* **Stubs** provide canned answers to calls made during the test,
  usually not responding at all to anything outside what's
  programmed in for the test.
* **Spies** are stubs that also record some information based
  on how they were called. One form of this might be an email
  service that records how many messages it was sent.
* **Mocks** are pre-programmed with expectations which form a
  specification of the calls they are expected to receive. They can
  throw an exception if they receive a call they don't expect and
  are checked during verification to ensure they got all the calls
  they were expecting.

## Further Reading

I expand on the use of Mocks, Doubles and the like in [Mocks Aren't Stubs](/articles/mocksArentStubs.html)

## Topics

[Architecture](/architecture)

[Refactoring](https://refactoring.com)

[Agile](/agile.html)

[Delivery](/delivery.html)

[Microservices](/microservices)

[Data](/data)

[Testing](/testing)

[DSL](/dsl.html)

## about me

[About](/aboutMe.html)

[Books](/books)

[FAQ](/faq.html)

## content

[Videos](/videos.html)

[Content Index](/tags)

[Fragments](/fragments)

[Board Games](/boardgames)

[Photography](/photos)

## Thoughtworks

[Home](https://thoughtworks.com)

[Insights](https://thoughtworks.com/insights)

[Careers](https://thoughtworks.com/careers)

[Radar](https://thoughtworks.com/radar)

[Engineering](https://www.thoughtworks.com/engineering)

## follow

[RSS](/feed.atom)

[Mastodon](https://toot.thoughtworks.com/@mfowler)

[LinkedIn](https://www.linkedin.com/in/martin-fowler-com/)

[Bluesky](https://bsky.app/profile/martinfowler.com)

[X](https://www.twitter.com/martinfowler)

[BGG](https://boardgamegeek.com/blog/13064/martins-7th-decade)

[![](/thoughtworks_white.png)](https://www.thoughtworks.com/engineering)

© Martin Fowler | [Disclosures](/aboutMe.html#disclosures)
