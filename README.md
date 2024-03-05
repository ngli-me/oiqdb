# OIQDB: an Opinionated Image Querying DataBase model

IQDB is a reverse image search system. It lets you search a database of images to find images that are visually similar to a given image.

This version of IQDB is a fork of the original IQDB used by https://iqdb.org, based on the (IQDB fork)[https://github.com/danbooru/iqdb/] used by Danbooru.

#### Reimplemented in Rust

Reimplementing the algorithm and features in Rust as a learning exercise.

(Make a diagram for here later)

#### TODO
<ul>
    <li> Implement proper response for the API </li>
    <li> Add sqlx integration </li>
    <li> Split up the haar transform methods so it can be tested more easily </li>
    <li> Add full testing </li>
    <li> Make it backwards compatible with the original (this *might* not be possible, it looks like the image reader in the original is corrupting the data or something) </li> 
</ul>

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
* https://hub.docker.com/repository/docker/evazion/iqdb
* https://unix4lyfe.org/haar/
* https://dsp.stackexchange.com/questions/58843/what-is-the-correct-order-of-operations-for-a-2d-haar-wavelet-decomposition

## Rust Stuff

### Axum

* Handling spawn blocking/bridging the sync and async portions of the code https://github.com/tokio-rs/axum/discussions/2045

Thanks to the Rust community for providing documentation and tutorials, the authors of the original algorithm, implementations and forks.
