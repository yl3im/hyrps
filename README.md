HYRPS - Hytera Radio Programming Software
======

This project allows users of Hytera radios to program them using a
cross-platform program written in Rust. It has been tested with the following
radios:

 - PD785G
 - X1p
 - MD785
 
Editing of the following codeplug features is currently supported:

 - Contact List
 - Digital Channels
 - Analogue Channels
 - Zones
 - Roam Lists
 - Scan Lists

Disclaimer
----
Use of this software is at your own risk; the program is in alpha and could
brick your radio! Be sure to take a firmware backup of your codeplug memory as
the first thing that you do. Boot your radio into firmware mode, connect
the programming cable and run:

``` console
$ hyrps fw-dump-cp-memory cp-backup.img
```

If you need to revert, you can restore by entering back into firmware mode,
connect the programming cable and running:

``` console
$ hyrps fw-write-cp-memory cp-backup.img
```

Printing out your codeplug
----

A good first place to start is printing out your codeplug:

``` console
$ hyrps print-codeplug

Digital Channels
================
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Name               TX Freq      RX Freq      Power   RX Only   TX Contact        Colour Code   Scan List   Timeslot   Vox   │
╞═════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╡
│ DMR CH 01          446.00625    446.00625    Low     false     1 World Wide      0             <None>      Slot1      false │
│ DMR CH 02          446.01875    446.01875    High    false     <None>            0             <None>      Slot1      false │

[...]
```

Creating a codeplug
----

Hyrps doesn't currently implement an interface to edit codeplugs outside of it's
software. You need to compile your own version which contains your own codeplug.
See `src/custom_cp.rs` and the function `mutate_cp` for an example. Once
complete, you can write that new codeplug to the radio with:

``` console
$ hyrps write-custom-codeplug
```

Advanced Features
-----

For anyone wanting to do some more advanced work with their codeplugs or perhaps
some codeplug image reverse engineering (contributions welcome!) the following
features may be useful:

### Reading and Writing Images

You can get a dump and write a codeplug image from the radio without having to
use firmware mode with the `dump-cp-memory` and `write-cp-memory` commands. This
can be useful since most of Hyrps' commands take an optional path to a codeplug
image which can be read as an alternative to reading from the radio.

### Dumping the section list

The Hytera codeplug image comprises of numerous sections, each of which has a
header, a payload and a mappings table. You can use the `print-sections` command
to traverse the sections and print out the headers:

``` console
$ hyrps print-sections

┌─────────────────────────────────────────────────────────────────────────────┐
│ Address    Type    Capacity   Elements in Use   Byte Size   Unk1   Unk2     │
╞═════════════════════════════════════════════════════════════════════════════╡
│ 0x392      0x2     0x1        0x1               0xB4        0x0    0xDAAF12 │
│ 0x462      0x3     0x1        0x1               0x30        0x0    0xDAAF12 │
│ 0x696      0x4     0x1        0x1               0x4         0x0    0xDAAF12 │
│ 0x6B6      0x5     0x1        0x1               0x8         0x0    0xDAAF12 │
│ 0x6DA      0x6     0x1        0x1               0x14        0x0    0xDAAF12 │
│ 0x70A      0x7     0x1        0x1               0x8         0x0    0xDAAF12 │
│ 0x72E      0x8     0x1        0x1               0x64        0x0    0xE2A962 │

[...]
```

### Dissecting a codeplug

As stated above, sections wrap one or more binary payloads. You can get hyrps to
dissect each section, apply the translation table and dump the binary payloads to a
file:

``` console
$ mkdir out
$ hyrps disect out
$ tree out
out/
├── 0x025C/
├── 0x025D/
│  └── 0000
├── 0x0253/
│  └── 0000
├── 0x0254/
│  ├── 0000
│  ├── 0001
│  ├── 0002
│  ├── 0003

[...]
```

Here each folder beginning with `0x` is the section type and the files under are
the in-use payloads.
