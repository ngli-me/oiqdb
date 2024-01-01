# OIQDB: an Opinionated Image Querying DataBase model

IQDB is a reverse image search system. It lets you search a database of images to find images that are visually similar to a given image.

This version of IQDB is a fork of the original IQDB used by https://iqdb.org, based on the (IQDB fork)[https://github.com/danbooru/iqdb/] used by Danbooru.

#### Reimplemented in Rust

Reimplementing the algorithm and features in Rust as a learning exercise.

# History

This version of IQDB is a fork of the Danbooru implementation of [IQDB](https://github.com/danbooru/iqdb/),
which is forked from the original [IQDB](https://iqdb.org/code).
The original code is written by [piespy](mailto:piespy@gmail.com). IQDB is based on code from
[imgSeek](https://sourceforge.net/projects/imgseek/), written by Ricardo
Niederberger Cabral. The IQDB algorithm is based on the paper
[Fast Multiresolution Image Querying](https://grail.cs.washington.edu/projects/query/)
by Charles E. Jacobs, Adam Finkelstein, and David H. Salesin.

OIQDB is distributed under the terms of the GNU General Public License, following the licensing of IQDB. See
[COPYING](./COPYING) for details.

# Further reading

* https://grail.cs.washington.edu/projects/query
* https://grail.cs.washington.edu/projects/query/mrquery.pdf
* https://cliutils.gitlab.io/modern-cmake/
* https://riptutorial.com/cmake
* https://github.com/yhirose/cpp-httplib
* https://hub.docker.com/repository/docker/evazion/iqdb
