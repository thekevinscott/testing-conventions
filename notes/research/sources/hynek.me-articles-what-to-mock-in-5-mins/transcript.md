---
url: https://hynek.me/articles/what-to-mock-in-5-mins/
title: “Don’t Mock What You Don’t Own” in 5 Minutes
fetched: 2026-06-27
raw: raw.html
transport: curl
capture_status: ok
---

[Hynek Schlawack](/)

[About Me](/about/)[Articles](/articles/)[TIL](/til/)[Talks](/talks/)

# “Don’t Mock What You Don’t Own” in 5 Minutes

21 June 2022

A common issue when writing tests for real-world software is how to deal with third-party dependencies. Let’s examine an old, but counter-intuitive principle.

Once upon a time, I made a stupid joke on Twitter about the *Don’t Mock What You Don’t Own* testing principle:

> Only mock what you own, because mocking others is rude.
>
> — Yours Truly,
> [Tweet](https://twitter.com/hynek/status/1478384282766913548)

While it didn’t get me fired, it led to me [being tricked](https://twitter.com/christianbarra/status/1478700652989722627) into giving a 5 minute-long talk[1](#fn:1) about it. Given the confused replies I got to the joke Tweet, I’ve decided it’s worth writing down the contents of the talk for posterity.

## The principle

*Don’t Mock What You Don’t Own* means that whenever you employ mock objects, you should use them to substitute your ***own*** objects and not third-party ones[2](#fn:2).

If you’re anything like me, that **makes no sense**! What else am I supposed to mock? My code is perfectly testable!

**Key point:** there’s a difference between owning an **object** and owning the **API** to use it.

Allow me to use a simple example to demonstrate the downsides of mocking third-party objects and what to do instead.

**Disclaimer**: I don’t like *mocks* in the original sense: a test object, that mimics another (usually complex) object, *records calls to its APIs*, and allows you to make assertions over those calls.

However, this principle is still useful, because it applies equally to any other type of test object. I will keep using the term *mock* throughout this article to remain congruent with the name of the principle and I will be using a popular mocking library in my examples for familiarity.

However, **I** don’t use mocks in my *own* code at all. In Python I use [*pretend*](https://github.com/alex/pretend) for simple stubs and *[verified fakes](https://pythonspeed.com/articles/verified-fakes/)* for more complex scenarios. In Go I reach always for *verified fakes*.

This isn’t *potayto potahto* – this is a fundamental difference in how to approach testing, but it’s out of scope for this article.

If you’re confused by the differences between *mocks*, *fakes*, *stubs*, *et cetera*, I recommend Martin Fowler’s *[Mocks Aren’t Stubs](https://martinfowler.com/articles/mocksArentStubs.html)*.

## A Docker repository client

To keep my example short, I will use a Python program that uses an HTTP library, but the problem and solution are universal to any object-oriented language.

If you’ve ever run your own Docker container registry, chances are high that you’ve written scripts around its [web API](https://docs.docker.com/registry/spec/api/). For example, you might want to print out a list of your repositories, together with a list of the tags that exist within each repository:

```
$ list-docker-repos-with-tags
web-svc 1, 5, 7
worker-svc 8, 10, 11
```

This is the inspiration for our example. If you don’t know what any of this means: don’t worry. All you need to understand is that we’re writing a program that makes HTTP requests against a web API and extracts data from the responses.

### Rude mocking

I’m going to skip a bit ahead and start with an already testable-looking function. It takes an HTTP client (in this case, I use the excellent [*httpx*](https://www.python-httpx.org) package) and returns [a dictionary](https://en.wikipedia.org/wiki/Hash_table) of repository names that point to a list of the version tags. Invoking this function from a CLI and printing out its return value is not interesting for our point, so I’m leaving it out:

```
def get_repos_w_tags(client):
    rv = {}
    repos = client.get(
        "https://docker.example.com/v2/_catalog"
    ).json()["repositories"]

    for repo in repos:
        rv[repo] = client.get(
            f"https://docker.example.com/v2/{repo}/tags/list"
        ).json()["tags"]

    return rv
```

First we fetch a list of repository names from the `_catalog` endpoint, then we iterate over them and fetch the list of tags for every repository.

And this is much better than a lot of code that gets written for situations like this! Many would hard-code everything and if they need to test it, they shrug their shoulders and start [monkey-patching](https://en.wikipedia.org/wiki/Monkey_patch). With the code above, you can “simply” pass in a fake `client` object that returns predefined static values for calls to `client.get()` and look at the dictionary it returns. *Look ma, no network!*

And yet it violates the principle that this article is about. Why?

Because it’s not simple. Let’s have a look at the simplest possible test that checks what happens if the first `client.get()` call returns an empty list for the `repositories` key:

```
from unittest.mock import Mock
import httpx

def test_empty():
    client = Mock(
        spec_set=httpx.Client,
        get=Mock(
            return_value=Mock(
                spec_set=httpx.Response,
                json=lambda: {
                    "repositories": []
                },
            )
        ),
    )

    assert {} == get_repos_w_tags(client)
```

We need **three** layers of mocks to verify that an empty `repositories` key leads to an empty dictionary. And if I didn’t use a `lambda` for the `json` function, it would be even four layers.

I use the `spec_set` argument ([as should you](https://nedbatchelder.com/blog/202202/why_your_mock_still_doesnt_work.html)) to prevent `unittest.mock.Mock` from happily returning new mocks when you access *any* attribute, and yet constructs like this tend to be brittle and unpleasant to create and debug.

This is a *business logic* test and the purpose of the test is drowning in boilerplate necessary to mimic the API of an HTTP client that can change at any time.

That makes the test **brittle** and **unidiomatic**. When I read tests for business logic, I want *the intent* of the test to be obvious on first glimpse.

You could write a helper to create mock clients that do what you want, saving you the boilerplate in individual tests. However, the more complex your tests get, and the more numerous and the more complex your involved mocks get, the less you can be sure what you’re *actually* testing. Eventually, you end up in [*Mock Hell*](https://www.youtube.com/watch?v=CdKaZ7boiZ4).

In my experience, this is the wrong path, so let’s try something different.

### Polite mocking

> All problems in computer science can be solved by another level of indirection.
>
> — Butler Lampson,
> [Fundamental Theorem of Software Engineering](https://en.wikipedia.org/wiki/Fundamental_theorem_of_software_engineering)

We’ll follow Mr. Lampson’s advice and add a *very thin* layer around the HTTP library, which becomes the [*façade*](https://en.wikipedia.org/wiki/Facade_pattern) between your clean code and the messy outside world. Layers like this are notoriously difficult to test[3](#fn:3), so they should be kept [cyclomatically](https://en.wikipedia.org/wiki/Cyclomatic_complexity) as simple as possible: go easy on conditionals and loops. Otherwise you just kick the proverbial testing can one layer down and win nothing.

In this case we write a `DockerRegistryClient` class that offers two methods whose implementations should look familiar: `get_repos()` that returns a list of repository names and `get_repo_tags()` that returns a list of tags for a repository. The code is the same as before:

```
class DockerRegistryClient:
    def __init__(self, client):
        self._client = client

    def get_repos(self):
        return self._client.get(
            "https://docker.example.com/v2/_catalog"
        ).json()["repositories"]

    def get_repo_tags(self, repo):
        return self._client.get(
             f"https://docker.example.com/v2/{repo}/tags/list"
        ).json()["tags"]
```

I hope there’s no surprises in this code, so let’s apply it to our business code:

```
def get_repos_w_tags_drc(drc):
    rv = {}
    for repo in drc.get_repos():
        rv[repo] = drc.get_repo_tags(repo)

    return rv
```

The first pay-off is that the business logic is more idiomatic! Seeing the business logic written clearly like this also reveals that it could be re-written as a dictionary comprehension! You never know beforehand what making code clearer and more idiomatic will yield you.

**This point can’t be overstated**: In an attempt to simplify our mocks by following a counter-intuitive principle from ancient times, we’ve *improved our business logic*.

Business logic is ultimately the reason you write software. It’s the reason an application exists. Having clean and idiomatic business logic pays dividends for as long as the software exists which is always longer than you think.

Let’s rewrite the test next:

```
def test_empty_drc():
    drc = Mock(
        spec_set=DockerRegistryClient,
        get_repos=lambda: []
    )

    assert {} == get_repos_w_tags_drc(drc)
```

Only one `Mock`! And *one* look and you know what’s happening!

Once you want to run more complex tests where `get_repos()` returns a non-empty list and you need to mock `get_repo_tags()` too, it’s a lot easier because all you have to do is adding another line with `get_repo_tags=lambda repo: …`. No need for more nested mocks and making `client.get()` return different values for different calls.

And if you choose to replace your HTTP library, your business tests won’t care, because they only interface with *your* abstraction.

**Corollary**: To keep your *business code* testable and idiomatic, avoid directly using third-party dependencies in it.

## When to break the rule?

Every rule and principle can be broken once you’ve fully understood its purpose. For example if an object already does have an idiomatic API, it’s probably not worth wrapping in an identical façade, just so it belongs to you.

For simple programs like in this blog post, it’s probably easier to write a test helper that creates appropriate fake HTTP clients. Although there’s the upside to the more idiomatic business code! At some point this becomes a trade-off against complexity and performance.

Sometimes it’s also easy enough to fake actual HTTP responses by running your own in-process HTTP server within the tests – but I prefer to isolate these kinds of tests when testing the *thin* outer layer. It also gets more complicated once you have to interact with opaque SOAP servers or CLI utilities. Trade-offs, trade-offs.

The most common occasion when **I** break this principle is when I need to simulate errors that aren’t trivial to create organically: certain network conditions, timeouts, integrity errors, …

**In the end, it’s less of a rule and more of a heuristic.**

## Further reading

To learn more about the principle check out [*That’s Not Yours*](https://8thlight.com/blog/eric-smith/2011/10/27/thats-not-yours.html) which takes a more database-centric approach and [this summary](https://github.com/testdouble/contributing-tests/wiki/Don't-mock-what-you-don't-own) in *TestDouble*’s wiki.

As with many things lately, this article was inspired by the wonderful [*Architecture Patterns with Python*](https://www.cosmicpython.com) – the book that keeps transforming my gut feelings into principles like none before. Go and read it (for free on the web page if you want to). Harry also wrote a much more extensive treatise on this topic entitled [*Writing tests for external API calls‌*](https://www.cosmicpython.com/blog/2020-01-25-testing_external_api_calls.html) that offers more ideas and approaches.

If you’re interested in object-oriented programming, subclassing, and Python, I’d like to recommend my *magnum opus*: [*Subclassing in Python Redux‌*](https://hynek.me/articles/python-subclassing-redux/)

Finally, this talk from 2012 has aged very well: *Stop Mocking, Start Testing*:

[Stop Mocking, Start Testing](https://youtube.com/watch?v=Xu5EhKVZdV8 "Play Video")

---

1. Hence the title of the post. No guarantees are given that you can read and comprehend this post in 5 minutes. [↩︎](#fnref:1)
2. It originates in the [*London School of TDD*](https://github.com/testdouble/contributing-tests/wiki/London-school-TDD) – the one that loooves mocks. [↩︎](#fnref:2)
3. My favorite tool for locally testing HTTP client libraries in Python is [*pytest-httpserver*](https://pypi.org/project/pytest-httpserver/).

   [*vcr*](https://github.com/vcr/vcr) and its many ports to various languages has many fans, too. I’m personally not one of them, because it works by monkeypatching HTTP clients and I prefer to keep my test code paths as close to production as possible.

   I find an in-process web server that returns canned responses much closer than magically replacing the client with a fake that runs entirely different code. [↩︎](#fnref:3)

This post was made possible by the [donations](/say-thanks/) from people and corporations who appreciate my public work.

Want more content like this? Here's my free, low-volume, non-creepy [*Hynek Did Something* newsletter](https://buttondown.com/hynek)!
It allows me to share my content directly with you and add extra context:

[![Hynek Schlawack](https://static.hynek.me/img/avatar_w200h200.jpg)](/about/)

Hynek Schlawack

Code Bohemian in ❤️ with Python 🐍, Go 🐹, and DevOps 🔧.
[Blogger](/articles/) 📝,
[speaker](/talks/) 📢,
[YouTuber](https://www.youtube.com/@The_Hynek) 📺,
PSF [fellow](https://www.python.org/psf/fellows-roster/) 🏆,
substance over flash 🧠.

Is my content helpful and/or enjoyable to you?
Please consider [supporting me](/say-thanks/)!
Every bit helps to motivate me in creating more.

© 2022
Hynek Schlawack

All Rights Reserved

•

[Impressum & Privacy](/legal/)
