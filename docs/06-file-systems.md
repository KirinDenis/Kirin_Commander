# 6. File systems

Kirin Commander's code never calls a system's file functions itself. It
calls one interface, `Fs`, and each place it runs gives one implementation
of it. A new place - a new storage in the browser, a network drive - is a
new implementation, and not a change anywhere in the commander.

## 6.1 Where it runs, and what is under it

| Place | Implementation | Underneath |
|---|---|---|
| DOS | `DosFs` | INT 21h (handles, `4Eh/4Fh` find first/next, `43h` attributes, `57h` dates, `36h` free space, `56h` rename), INT 24h taken over for critical errors, INT 25h/26h never |
| Windows | `WinFs` | Win32: `FindFirstFileExW`, `CreateFileW`, `MoveFileExW`, `CopyFileExW` (with its progress callback), `GetFileAttributesExW`, `SHFileOperationW`/`IFileOperation` for the Recycle Bin, `GetDiskFreeSpaceExW`, `GetLogicalDrives`/`GetVolumeInformationW` |
| Linux | `PosixFs` | `opendir/readdir`, `stat/lstat`, `open/read/write`, `rename`, `copy_file_range` (else read/write), `statvfs`, `/proc/mounts` for the drive menu, the freedesktop Trash for delete |
| Browser | `WebFs` | one per drive: OPFS (the site's `BrowserStorage`), a folder of the computer (File System Access), the server's folder, WebDAV, the read-only examples, the running DOS PC's disk - the sources OWLOSUI and WIRE CITY already have |

## 6.2 The interface

What a panel and an operation need, and nothing a single system happens
to have:

```
drives()                      -> [Drive { name, kind, label, free, total, ready, readOnly }]
list(dir)                     -> [Entry { name, short?, kind: file|folder|link, size, modified, attrs }]
stat(path)                    -> Entry | NotFound
read(path, offset, count)     -> bytes
open_write(path, mode)        -> Writer      mode: create-new | replace | append
  Writer.write(bytes), Writer.finish(), Writer.abandon()
mkdir(path)
remove(path)                  -> a file, or an EMPTY folder
rename(from, to, replace)     -> same volume only: CrossVolume otherwise
copy(from, to, replace)?      -> optional: a system copy (keeps dates, is fast); else the commander copies by read/write
set_attrs(path, attrs, modified)?
trash(path)?                  -> optional: to the bin
free(path)                    -> bytes free on the volume, or Unknown
same_volume(a, b)             -> bool
watch(dir, f)?                -> optional: told when the folder changes
caps                          -> what it can do: attrs, dates, links, trash, case-sensitive, long names, max name
```

- **Paths** inside the commander are one form: a drive and a list of
  names (`C:` + `GAMES`, `OWLFLY3`). Each `Fs` turns it into its own:
  `C:\GAMES\OWLFLY3`, `/home/me/games/owlfly3`, `/DOS C Drive/GAMES/OWLFLY3/`.
  On Linux the "drives" are `/`, the home folder and the mounted volumes.
- **`remove` is not recursive.** A folder with files in it is the
  commander's to empty, file by file, so that each file can be asked
  about, counted, and reported on error - the same on every system.
- **`open_write` with `create-new`** fails with `Exists` rather than
  overwriting: the question "overwrite?" is asked by the commander, never
  answered by a system's default.
- **Case**: `caps.case_sensitive` says whether `README` and `readme` are
  two files. The commander compares names with it, not with its own
  assumption. (OPFS is case-sensitive; DOS and Windows are not; WIRE CITY
  spells DOS names against OPFS the case-blind way and that stays.)

## 6.3 One model for slow and quick systems

On DOS and a desktop a file call returns when it is done. In a browser
every one is a promise. The commander is written once, for both:

- Every operation is a **job**: started, it gives back an id at once;
  the UI loop asks it for **events** - `progress(bytes, files)`,
  `question(kind, ...)`, `error(kind, path, code)`, `done(summary)`.
- On DOS a job runs a slice of work per turn of the loop (a few K of a
  copy), so the screen stays alive and Esc is heard - DOS has no threads.
- On a desktop a job runs on a worker thread and posts events.
- In the browser a job is an async function whose awaits post events.
- A job waiting on a `question` does nothing until it is answered: the
  overwrite question, an error's Retry/Skip, Stop.

The panels themselves (`list`, `stat`) are jobs too, so a slow network
folder shows "Reading..." in the panel instead of freezing it.

## 6.4 Errors: one table of kinds

Each `Fs` turns its system's errors into these kinds. The commander's
words in [05-file-operations.md](05-file-operations.md) section 5.8 are
written once, per kind.

| Kind | DOS (INT 21h / INT 24h) | Windows | Linux | Browser |
|---|---|---|---|---|
| NotFound | 02h | `ERROR_FILE_NOT_FOUND` 2 | `ENOENT` | `NotFoundError`; HTTP 404 |
| PathNotFound | 03h | `ERROR_PATH_NOT_FOUND` 3 | `ENOENT` (on a folder of the path), `ENOTDIR` | `NotFoundError` (a folder of the path) |
| Exists | 50h | `ERROR_FILE_EXISTS` 80, `ERROR_ALREADY_EXISTS` 183 | `EEXIST` | (checked before writing: OPFS has no create-new) |
| AccessDenied | 05h | `ERROR_ACCESS_DENIED` 5 | `EACCES`, `EPERM` | `NotAllowedError`, `SecurityError`; HTTP 401/403 |
| ReadOnlyFile | 05h on a file with attribute R | `ERROR_ACCESS_DENIED` + `FILE_ATTRIBUTE_READONLY` | `EACCES` on a file without w | - |
| ReadOnlyVolume | INT 24h 00h (write-protected) | `ERROR_WRITE_PROTECT` 19 | `EROFS` | a drive mounted read-only; the examples |
| NotReady | INT 24h 02h; 15h | `ERROR_NOT_READY` 21 | `ENOMEDIUM` | "No DOS machine is running" (the DOS PC's disk) |
| DiskFull | 27h; a write that wrote fewer bytes (DOS 3 says nothing else) | `ERROR_DISK_FULL` 112, `ERROR_HANDLE_DISK_FULL` 39 | `ENOSPC`, `EDQUOT` | `QuotaExceededError` |
| NotEmpty | 05h on rmdir; 10h for the current folder | `ERROR_DIR_NOT_EMPTY` 145 | `ENOTEMPTY` | `InvalidModificationError` |
| IsFolder / IsFile | 05h | `ERROR_DIRECTORY` 267 | `EISDIR` / `ENOTDIR` | `TypeMismatchError` |
| InUse | 20h sharing, 21h lock (with SHARE) | `ERROR_SHARING_VIOLATION` 32, `ERROR_LOCK_VIOLATION` 33 | `EBUSY`, `ETXTBSY` | `NoModificationAllowedError` (a file open elsewhere) |
| BadName | 0Bh; an 8.3 rule broken (checked before calling) | `ERROR_INVALID_NAME` 123 | `EINVAL` | a name OPFS refuses (`TypeError`) |
| NameTooLong | (8.3, checked before) | `ERROR_FILENAME_EXCED_RANGE` 206 | `ENAMETOOLONG` | - |
| CrossVolume | 11h | `ERROR_NOT_SAME_DEVICE` 17 | `EXDEV` | two drives |
| TooManyOpen | 04h | `ERROR_TOO_MANY_OPEN_FILES` 4 | `EMFILE`, `ENFILE` | - |
| Media | INT 24h 04h CRC, 0Ah write fault, 0Bh read fault, 0Ch general failure; 17h, 1Dh, 1Fh | `ERROR_CRC` 23, `ERROR_GEN_FAILURE` 31 | `EIO` | - |
| Network | 35h-58h network errors | `ERROR_NETWORK_*`, `ERROR_BAD_NETPATH` 53 | `ENETDOWN`, `EHOSTUNREACH`, `ETIMEDOUT` | the server or the share did not answer; CORS |
| Unsupported | 01h | `ERROR_NOT_SUPPORTED` 50 | `ENOTSUP`, `EOPNOTSUPP` | a source without rename, attributes |
| Interrupted | (Esc in the commander) | `ERROR_REQUEST_ABORTED` 1235 | `EINTR` | `AbortError` |
| Other | anything else, with its code | | | |

- On DOS the kind comes from INT 21h 59h ("get extended error") after the
  call failed; the critical error (INT 24h) is caught by the commander's
  own handler, which records it and answers "Fail", so the call returns
  with an error the job then asks about. DOS's "Abort, Retry, Fail?" never
  appears over the commander's screen.
- A write that wrote fewer bytes than asked is **DiskFull**, on every
  system, whatever the system said.
- The system's own code and words travel with the kind, for the small
  text at the end of the error box and for the log.

## 6.5 The browser, more closely

The sources the site already has (`list`, `read`, `readBytes`, `write`,
`remove`, `mkdir`, `rename`; folders end with `/`) are the start of
`WebFs`. What the commander needs beyond them, per source:

- `stat` and `exists` - from `list` of the parent where a source has
  nothing better.
- `open_write` in parts with `create-new` - OPFS's `createWritable` with
  `keepExistingData: false`, checked for the name first; written under a
  temporary name and moved into place at `finish`.
- `rename` - OPFS `handle.move` where the browser has it; otherwise the
  source says `Unsupported` and the commander copies and removes, as
  across volumes.
- `remove` non-recursive - `removeEntry` without `{recursive: true}`,
  which fails on a full folder: `NotEmpty`.
- `free` - `navigator.storage.estimate()`, for every OPFS drive together.
- The site's journal of removals and its read-only drives (`Drive.guard`)
  stay what they are: `WebFs` goes through the site's `Drive`, not around it.
- The DOS PC's own disk (js-dos `fsTree`, `fsReadFile`, ...) has no rename
  and no dates: `caps` says so, and the commander copies-and-removes and
  shows no date.

## 6.6 What the commander checks itself, everywhere

- A name against the target's rules before anything is written (8.3 on
  DOS; reserved names `CON`, `PRN`, `NUL`, `AUX`, `COM1`.. on DOS and
  Windows; `/` and NUL everywhere).
- Copy onto itself, a folder into itself, by comparing the resolved paths
  with the file system's own case rule.
- The space a copy needs against `free`, before starting - a warning with
  "Copy anyway?", since `free` can be wrong (network, compression).
