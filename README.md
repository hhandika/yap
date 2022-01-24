# YAP (Yet-Another-Pipeline)

![YAP-Tests](https://github.com/hhandika/yap/workflows/YAP-Tests/badge.svg)
[![Build Status](https://www.travis-ci.com/hhandika/yap.svg?branch=main)](https://www.travis-ci.com/hhandika/yap)

A pipeline for processing high-throughput sequencing data. The goals:

1. To develop single contained executable pipeline that reduce the numbers of dependecies at runtime.
2. Robust error handlings.
3. Elegant commands.
4. Fast

Under development. More soon!

## Features

Below are the planned and already implemented features:

| Features                                      | Dependencies                                       | Implementation |
| --------------------------------------------- | -------------------------------------------------- | -------------- |
| _Essentials_                                  |
| Batch adapter trimming and sequence filtering | [Fastp](https://github.com/OpenGene/fastp)         | Done           |
| Batch sequence assembly                       | [SPAdes](https://github.com/ablab/spades)          | Done           |
| Sequence statistics                           | None                                               | Done           |
| Read mapping                                  | [BWA-MEM2](https://github.com/bwa-mem2/bwa-mem2)   | Planned        |
| Sequence alignment                            | [Mafft](https://mafft.cbrc.jp/alignment/software/) | Planned        |
| Alignment trimming                            | [TrimAl](http://trimal.cgenomics.org/introduction) | Planned        |
| Alignment format conversion                   | [ReadAl](http://trimal.cgenomics.org/introduction) | Planned        |
| _Utilities_                                   |
| Sequence finder                               | None                                               | Done           |
| Sequence file renamer                         | None                                               | Planned        |
| _Extras_                                      |
| Logger                                        | None                                               | Planned        |
| Symlink fixer                                 | None                                               | Planned        |

## Installation

YAP operating system support:

1. MacOS
2. Linux
3. Windows-WSL

Everyone is welcome to try the development version of YAP. First, please install [the Rust Compiler](https://www.rust-lang.org/learn/get-started), and then:

```{Bash}
cargo install --git https://github.com/hhandika/yap
```

Confirm the app properly installed:

```{Bash}
yap --version
```

It should show the app version.

## Usages

Current working commands:

```{Bash}
USAGE:
    yap <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    assembly    Assemble reads using SPAdes
    check       Checks dependencies
    help        Prints this message or the help of the given subcommand(s)
    new         Find sequences and generate input files
    qc          Trims adapters and clean low quality reads using fastp
    stats       Get sequence statistics

```

More about the subcommands:

```{Bash}
yap <SUBCOMMAND> --help
```

## Workflow

### Step 1. Generate a configuration file

The first step for working on yap is to generate the config file for subsequent analyses. The config file is just a simple .conf or .csv file that contain the sequence name for the clean-read folders and the path to the raw read. In some pipelines, you may do this manually. In yap, the app write it for you. It is simplify and speed-up the workflow and also avoid typing errors when it is done manually. By ,default, yap will generate .conf file The command is as below:

```Bash
yap new -d [directory-to-raw-read-fastq-files]
```

Yap infer the sequence name from the raw reads file name. By default, it looks for the file name with this pattern:

```Bash
genus_species_voucherNo_readNo.extension
```

For example, our raw files are in the folder `/home/users/test_uce/`. The folders contains two sequences with two reads each:

```Bash
test_files/
├── Bunomys_andrewsi_museum6789_locality1_READ1.fq.gz
├── Bunomys_andrewsi_museum6789_locality1_READ2.fq.gz
├── Bunomys_chrysocomus_museum12345_locality1_READ1.fq.gz
└── Bunomys_chrysocomus_museum12345_locality1_READ2.fq.gz
```

When we run the `yap new` in the directory above using the default settings, the resulting .conf file will be as below:

```Bash
[seqs]
Bunomys_chrysocomus_museum12345:/home/users/test_uce/
Bunomys_andrewsi_museum6789:/home/users/test_uce/
```

If you prefer to capture the locality name from the file, you can change the default word length 3 `--len` or `-l` to 4. The command will be as below:

```Bash
yap new -d [raw-read-dir] -l 4
```

The resulting .conf file will be as below:

```Bash
[seqs]
Bunomys_chrysocomus_museum12345_locality1:/home/users/test_uce/
Bunomys_andrewsi_museum6789_locality1:/home/users/test_uce/
```

If you prefer to generate the configuration file in csv. You can pass the flag `--csv`.

### Step 2. Cleaning raw sequence reads using Fastp

To clean the read, we only need to feed yap with the configuration file we generate in step 1:

```Bash
yap qc -i yap-qc_input.conf
```

To check if `yap` parse the input file correctly. You can do:

```Bash
yap qc -i yap-qc_input.conf --dry
```

You can also pass Fastp parameters using `--opts=` option and put fastp params in quotation. The code implementation allows you to pass any Fastp paremeter available now and in the future.

### Step 3. Assembly clean sequence reads using SPAdes

If you clean your reads using `yap` workflow. You only need to do assembly using the auto settings.

```Bash
yap assembly auto
```

You can pass SPAdes parameter using `--opts=` option. The code implementation allows you to pass any SPAdes paremeter available now and in the future.

```Bash
yap qc auto --opts="SPAdes-params"
```

## State of Code

All implemented features are working as expected. Please, expect significant code changes as the development of the program is still at the early stage.
