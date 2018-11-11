## libimagentryref

This library crate contains functionality to generate _references_ within the
imag store.

### Problem

The problem this library solves is the following: A user wants to refer to a
file which exists on her filesystem from within imag.
But unfortunately, the user has several devices and the filesystem layout (the
way the $HOME is organized) is not the same on every device.
With this library, the user is able to refer to a file, but without specifying
the whole path.

Each device can have a different "base path", files are re-found via their
hashes and file names, assuming that the files are equal on different devices or
have at least the same name.


### User Story / Usecase

Alice has a music library on her workstation and on her notebook. On her
workstation, the music collection is at `home/alice/music`, on the notebook, it
exists in `/home/al/media/music`.

From within imag, alice wants to create a link to a file
`$music_store/Psy_trance_2018_yearmix.mp3`.

`libimagentryref` helps her, because she can provide a "base path" in the
imag configuration file of each device and then link the file. imag only stores
data about the file and its relative path, but not its abolute path.

When moving the imag store from the workstation to the notebook, the base path
for the music collection is not `/home/alice/music` anymore, but
`/home/al/media/music` and imag can find the file automatically.


### Solution, Details

libimagentryref does store the following data:

```toml
[ref]
filehash.sha1 = "<sha1 hash of the file>"
relpath = "/Psy_trance_2018_yearmix.mp3"
```

The filehash is stored so that libimagentryref can re-find the file whenever it
was moved. The `sha1` key is added to be able to upgrade hashes later to other
hashing algorithms.
`relpath` is the part of the path that when joined with the "base" path from
the configuration results in the full path of the file for the current machine.

libimagentryref provides functionality to get the file. 
libimagentryref also offers functionality to find files _only_ using their
filename or filehash and correct the filehash or filename respectively
(automatically or explicitely).


### Limits

As soon as the file is renamed _and_ modified, this fails.
This does also not cover the use case where the same file has different names on
different machines.


