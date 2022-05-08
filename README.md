# simpledb

## Simple Database Challenge by Thumbtack

Simple key/value store with nested transactions and value counting, implemented in roughly 200 lines of Rust. The original [page](http://www.thumbtack.com/challenges/simple-database) seems to be gone, but this [wiki](https://gitlab.com/nehaleadz/Simple-Database-Challenge/-/wikis/home) provides a good overview.

This application lacks the command parser as that is not interesting, but when run will show a quick demo of transaction support. The equivalent functions are `set()`, `get()`, `delete()`, and `count()` (`SET`, `GET`, `UNSET`, and `NUMEQUALTO` respectively). The transaction functions are `begin()`, `commit()`, and `rollback()`.

## Prior Art

These are just a few examples, there may be more.

* Java version: https://github.com/dianapojar/simpledatabase
* Python version: https://gitlab.com/nehaleadz/Simple-Database-Challenge
