# Rust-GA live stream planning <!-- omit in toc -->

I'm going to use this to do some planning and documentation for the
live stream. Not sure how well that will work out, but just putting
`TODO`s in the code doesn't always provide any kind of "big picture".

I could do a lot of this using GitHub Issues, and just use this
as a "diary" document to track what happened in each session? I
guess it depends a lot on how many items make it into the "Issue
to address" list; if there are only a few keeping them here probably
works well, but if there get to be a lot of them moving to GitHub
Issues might make more sense.

- [Issues to address](#issues-to-address)
  - [Use `clap` to support command-line args for parameters](#use-clap-to-support-command-line-args-for-parameters)
  - [Implement lexicase selection](#implement-lexicase-selection)
  - [Replace `Individual` with traits?](#replace-individual-with-traits)
  - [Create mutation/recombination pipeline](#create-mutationrecombination-pipeline)
  - [Supported weighted parent selection](#supported-weighted-parent-selection)
  - [Move `scorer` inside `Generation`?](#move-scorer-inside-generation)
- [Wednesday, 21 Sep 2022 (7-9pm)](#wednesday-21-sep-2022-7-9pm)
  - [What actually happened](#what-actually-happened)

## Issues to address

Below are some things that we could/should deal with, in no
particular order.

### Use `clap` to support command-line args for parameters

Using `clap` in [the Rust echo client-server](https://github.com/NicMcPhee/rust-echo-client-server)
was really easy, and worked quite nicely. We should add `clap`
support for the various parameters that we might want to
vary via the command line. At the moment that would include:

- Population size
- Length of bitstrings
- Selectors
- Mutation/recombination tools
- Target problem

Some of these may depend on others. The length of bitstrings
makes sense here because that's the only genome we're using, but
if we add other genomes (like tree-based GP or Push) then that
won't make any sense.

I'm not sure, but it seems possible that this kind of
interdependence may create type problems for us in Rust. Maybe
traits would be the way to address this?

### Implement lexicase selection

I would really like to try the HIFF problem with lexicase selection,
which requires actually implementing lexicase selection.

This is going to require extending `Individual` so that it
contains a vector of errors and not just total error. I'll also
need to change the name of the `error` field to `total_error`.

Should this be a trait in some way? Should there be a trait that
"requires" a vector of errors and we implement lexicase selection
on a `Vector` of things that satisfy that trait? Then we could
(I think) say things like "You can use lexicase selection on a
population of things of type `T` where `T` implements the
`HasErrorVector` trait"?

### Replace `Individual` with traits?

I'm not sure about this, but ["Implement lexicase selection"](#implement-lexicase-selection) makes me wonder if we the
`Individual` type even makes sense, and whether it should be
replaced by one or more traits that specify that various
attributes that different kinds of `Individual`s should have.

If you look at `Population`, e.g., currently that only makes
two assumptions about `Individual`:

- We can construct one, which I think the `Default` trait would
  take care of for us, and
- We can call `.score()`, which a new `HasScore` or `Scorable`
  or whatever trait could take care of.

Then when we add lexicase selection that could require that the
individuals implement the `HasErrorVector` trait or similar. And
when we get into GP, we might have traits like `HasTree` and
`HasPlushy`.

I'm not entirely sure how this all would work out, but I feel
like this would be the more Rust-y (and ultimately more flexible)
approach than having a "fixed" `Individual` type. This would
be something like what I did with the `LinearCrossover` and
`LinearMutation` traits.

### Create mutation/recombination pipeline

I think we have the parent selection thing in a reasonable
place, but still we need to work out pipelines of mutation
and recombination operators.

The addition of a closure that captures most of this pipeline
in the construction of a `Generation` type may essentially
resolve this issue, or at least suggests a way to approach it.

### Supported weighted parent selection

We should extend `PopulationSelector` to a `WeightedParentSelector` that
is essentially a wrapper around `rand::distributions::WeightedChoice`
so we can provide weights on the different selectors.

### Move `scorer` inside `Generation`?

Should the `scorer` be inside the generation so we don't have to
  keep capturing it and passing it around?

Or should there actually be a `Run` type (or a `RunParams` type)
that holds all this stuff and is used to make them available to
types like `Generation` and `Population`?

## Wednesday, 21 Sep 2022 (7-9pm)

Two things that would be good to do that would follow up on work
we did last week:

- Should the new `PopulationSelector` type just become part of
  the `Population` type?
  We could provide a set of selectors in the constructor (or
  a builder) for `Population`, and then just have a `get_parent()`
  method there.
- Extend `PopulationSelector` to a `WeightedParentSelector` that
  is essentially a wrapper around `rand::distributions::WeightedChoice`
  so we can provide weights on the different selectors.
- If those get done quickly, then we can look at the problem of
  pipelining mutation and recombination operators.

### What actually happened

`esitsu@Twitch` suggested moving the `ParentSelector` logic into a new
`Generation` type, which would then have a `next()` method to generate
the next generation from the current one. We then spent pretty much the
entire stream implementing this (good) idea.

Most of it was pretty straightforward, but we got really bogged down at
the end because of a problem with the Rust compiler's understanding of a
closure's types.

`esitsu` also had a really cool idea of extracting the bit string specific
parts of the `next` generation logic into a closure that we'd pass in when
we construct the generation. This worked, but I didn't explicitly type the
arguments to the closure initially, and that let to all kinds of weird
confusion. We flailed pretty hard trying to resolve the issues, doing all
sorts of things with lifetimes. In the end it turned out that all we
_really_ needed to do was explicitly type that closure, and once we did
that everything worked.

After the stream ended I removed the `next_generation()` logic and
`ParentSelector` type from `Population`, moving the former into
`Generation` and deleting the latter altogether.
