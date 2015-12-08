# gringotts
A database written in Rust.

# Disclaimer
This is still heavily under construction (both design and implementation) and is not useful to anyone at this point.

# Goals
Primary goals include:
* The database should work equally well in single user (ie embedded) and multi-user modes (like PostGres and most SQL databases).
* The database should be fast.
* The database codebase should be simple.
 * I want to be able to keep it's structure in my head at one time and be able to teach it in (much) less than day.  Such as over a beer at lunch.
* The database should not be at risk for corruption
 * This goes along with "simple".  A simple code structure with single entry points into actions should prevent corruption, which often occurs due to multiple threads not playing nice with each other.
* There should be nothing that requires a restart.  The system should be able to adapt and make any requested changes on the fly, even if it costs a little performance.
* The design should always be flexible and forward focused, to make it easier to expand later without breaking changes.
** For instance, designing a variable length encoding like UTF-8, even if we only need to use something like ASCII now.  Doing that would ensure we could expand the encoding later without requiring a conversion of all of those original encodings to use a new version of the database.

# Architecture
**WARNING: The following is a tentative plan.  I make no promises to stick to any of it yet.**
The plan is for this database to have two layers: an underlying key-value store, and an overlaying object model.  Gringotts refers to the overlaying database, not the key-value store, and my plan is to hide the key-value store implementation from public APIs by v1.

## Vaults
At the highest layer, data in Gringotts will be grouped into "vaults".  These vaults are roughly equivalent to "tables" in other databases.  There are two types of vaults, record style, and hierarchial style.  

Hierarchial style vaults are similar to other key-value stores, where there is no overarching schema to the data.  Changes are fully logged with timestamps, and the state of teh database can be found for any given time.  This is usefull for storing data such as settings, which don't change often and don't have a repetative structure.

Record style vaults are more like tradiditonal table based layouts.  A schema will be defined for each record vault, with fields for each record.   Each record will be indexed by at least a machine-generated key, which can be used to uniquely identify a record.

### Enforcer Scripts
For each vault, there will be optional enforcer scripts that will able to be written in just about any language.  The enforcer script performs validation on new records and record updates, and can reject records based on their contents.  They should be incredibly flexible, although I don't plan for them to be used extensively, as they can confuse application code. ***This is one of the more dangerous aspects to Gringotts.  Although they can be very useful, I fear that these could be abused very easily.  I may need to think more about their design.***

# Nice-To-Haves
(None yet).

# Plan
The plan below is mostly to ensure that things work and my work is done in small, manageable chunks.  It is also so that I can have something functional to write tests against very early on.  This means that things will be implemented often once early on in a very inefficient way that doesn't scale, and re-implemented later using a sane design.  

v1 is the target at which I would consider the database to have accomplished all of the goals and design above, minus nice-to-haves that come up during the implementation.  At that point, the entire planned API will be complete and existing public interfaces will be promised for the rest of the major version.

v0.1 is the target for having basic vaults up and running.

Prior to v1, there will be far more minor releases than after, and the following development scheme may not be followed to a tee, due to the fact that nothing is promised to be stable:
* Major Versions
 * Breaking changes.  All breaking changes should be deprecated for at least one major version before being removed.
* Minor Versions
 * Non-breaking features.
* Build Versions
 * Bug fixes

_v0.0.1_
* At this point, we should have full key-value access.  It will be implemented using a single file that has a header and a body.  The entire body will be read and interpreted into a hash.
* There will be support for different database files at the library level, but the control binaries will have that hard-coded for them.
* `kvctl` will be implemented as a binary with the ability to set and retrieve keys/values.
 * `kvctl get [key]`
   * echoes the value to stdout
 * `kvctl set [key] [value]`
   * echoes the old value to stdout
* Failures will just result in a panic.  There will be no intercepting of errors.

_v0.0.2_
* `dbctl` will be implemented as a binary with the ability to create, verify, and delete dbfiles. 
* `kvctl` will now look for environment variable `GRINGOTTS_DBFILE` or use `--dbfile` override to identify which file it is working on.
 * If neither is specified, it will use (and create if necessary) a default file called `//data/default.gdb`.
