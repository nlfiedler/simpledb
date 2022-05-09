# simpledb

# Simple Database Challenge by Thumbtack (in Rust)

An implmenetation of the Simple Database Challenge in roughly 200 lines of Rust. The original [page](http://www.thumbtack.com/challenges/simple-database) seems to be gone, but this [wiki](https://gitlab.com/nehaleadz/Simple-Database-Challenge/-/wikis/home) provides a good overview. In short, it is a simple in-memory key/value store with support for nested transactions, with the additional feature of counting the number of occurrences of a particular value (a reverse index).

This application lacks the command parser as that is not interesting, but when run will show a quick demo of transaction support. The functions and their equivalent commands are shown below.

| Function     | Command      |
| ------------ | ------------ |
| `set()`      | `SET`        |
| `get()`      | `GET`        |
| `delete()`   | `DELETE`     |
| `count()`    | `NUMEQUALTO` |
| `begin()`    | `BEGIN`      |
| `rollback()` | `ROLLBACK`   |
| `commit()`   | `COMMIT`     |

## Prior Art

These are just a few examples, there may be more.

* Java version: https://github.com/dianapojar/simpledatabase
* Python version: https://gitlab.com/nehaleadz/Simple-Database-Challenge
