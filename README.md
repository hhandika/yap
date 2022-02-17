# YAP (Yet-Another-Pipeline)

![YAP-Tests](https://github.com/hhandika/yap/workflows/YAP-Tests/badge.svg)
[![Build Status](https://www.travis-ci.com/hhandika/yap.svg?branch=main)](https://www.travis-ci.com/hhandika/yap)
[![DOI](https://zenodo.org/badge/325113546.svg)](https://zenodo.org/badge/latestdoi/325113546)

A pipeline for processing high-throughput sequencing data. The goals:

1. To develop single contained executable pipeline that reduce the numbers of dependecies at runtime.
2. Robust error handlings.
3. Elegant commands.
4. Fast

Yap is a part of our phylogenomic toolkit that includes a wrapper to easily generate gene and species trees ([`myte`](https://github.com/hhandika/myte)), and alignment statistics and manipulation tool ([`segul`](https://github.com/hhandika/segul)).

> CITATION: Heru Handika. (2022). YAP: A single executable pipeline for phylogenomics (v0.3.0). Zenodo. https://doi.org/10.5281/zenodo.6128874

## Features

Below are the planned and already implemented features:

| Features                                      | Dependencies                                       | Implementation                                                        |
| --------------------------------------------- | -------------------------------------------------- | --------------------------------------------------------------------- |
| _Essentials_                                  |
| Batch adapter trimming and sequence filtering | [Fastp](https://github.com/OpenGene/fastp)         | Done                                                                  |
| Batch sequence assembly                       | [SPAdes](https://github.com/ablab/spades)          | Done                                                                  |
| Sequence statistics                           | None                                               | Done                                                                  |
| Read mapping                                  | [BWA-MEM2](https://github.com/bwa-mem2/bwa-mem2)   | Planned                                                               |
| Sequence alignment                            | [Mafft](https://mafft.cbrc.jp/alignment/software/) | Planned                                                               |
| Alignment trimming                            | [TrimAl](http://trimal.cgenomics.org/introduction) | Planned                                                               |
| Alignment format conversion                   | None                                               | Cancelled (implemented in [segul](https://github.com/hhandika/segul)) |
| _Utilities_                                   |
| Sequence finder                               | None                                               | Done                                                                  |
| Sequence file renamer                         | None                                               | Planned                                                               |
| _Extras_                                      |
| Logger                                        | None                                               | Planned                                                               |
| Symlink fixer                                 | None                                               | Planned                                                               |

## Installation

YAP operating system support:

1. MacOS
2. Linux
3. Windows-WSL

The quickest way to install the app is using a pre-compiled binary available in [the release page](https://github.com/hhandika/yap/releases). Download, extract, and copy the file to your PATH environment to be able to run it in any working directory in your system.

You can also install segul by compiling it from the source. First, please install [the Rust Compiler](https://www.rust-lang.org/learn/get-started), and then:

```{Bash}
cargo install --git https://github.com/hhandika/yap
```

Confirm the app properly installed:

```{Bash}
yap --version
```

It should show the app version.

### Dependencies

Read cleaning: Fastp ([INSTALL](https://github.com/OpenGene/fastp#get-fastp))

Assembly: SPAdes ([INSTALL](https://cab.spbu.ru/software/spades/))

Statistic: None

To check if yap detect the dependencies:

```Bash
yap check
```

It will show your system information and the dependency status:

```Bash
System Information
Operating system        : Ubuntu 20.04
Kernel version          : 4.19.104-microsoft-standard
Available cores         : 4
Available threads       : 8
Total RAM               : 7 Gb

Dependencies:
[OK]    fastp 0.20.0
[OK]    SPAdes v3.13.1
```

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

The first step for working on yap is to generate a config file for subsequent analyses. The config file is a .conf or .csv file that contain the sequence name for the clean-read folders and the path to the raw read. In some pipelines, you may do this manually. In yap, the app write it for you. It simplifies and speeds-up the workflow and also avoid typing errors when it is done manually. By default, yap will generate .conf file The command is as below:

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

An option to use a configuration file is also available. You can use a two-column csv:

| Samples         | Path                                       |
| --------------- | ------------------------------------------ |
| some_species    | clean_reads/some_species/trimmed_reads/    |
| another_species | clean_reads/another_species/trimmed_reads/ |

Or using ini format:

```Bash
[samples]
some_species:clean_reads/some_species/trimmed_reads/
another_species:clean_reads/another_species/trimmed_reads/
```

Then, save your configuration file. The extension for your file does not matter, you could just save it as txt. The command to run spade-runner using a configuration file is as below:

```Bash
yap assembly conf -i [path-to-your-config-file]
```

For example

```Bash
yap assembly conf -i bunomys_assembly.conf
```

You can check if the app correctly detect your reads using the `dry-run` option:

```Bash
yap assembly auto -d [your-clean-read-folder] --dry
```

or

```Bash
yap assembly conf -i [path-to-your-config-file] --dry
```

By default, the app passes `--careful` options to SPAdes. The full command is equal to running SPAdes using this command:

```Bash
spades --pe1-1 [path-to-read1] --pe1-2 [path-to-read2] -o [target-output-dir] --careful
```

It will add `--pe1-s [path-to-singleton/unpaired-read]` if the app detects a singleton read in your sample directory.

You can also specify the number of threads by passing `-t` or `--threads` option:

```Bash
yap assembly auto -d [your-clean-read-folder] -t [number-of-threads]
```

or if you use a config file:

```Bash
yap assembly conf -i [path-to-your-config-file] -t [number-of-threads]
```

Other SPAdes parameter is available by using `--opts` option. For example, here we define max memory size to 16 gb. The program will override the careful option used in the default settings. Hence, we will need to pass it again if we want to use it.

```Bash
yap auto -d clean_reads/ --opts="--careful -m 16"
```

The app won't check the correctness of the parameters. Instead, it will let SPAdes checking them. This way it gives user flexibility to pass any SPAdes cli parameters available for pair-end reads.

You may not want to keep all the resulting SPAdes files. To clean the resulting files:

```Bash
yap assembly clean -d [assembly-dir]
```

Yap will only retain these files:

```Bash
/contigs.fasta
/scaffolds.fasta
/spades.log
/warnings.log
```

### Optional Step. Generate sequence statistics

`yap` provide a fast summary statistics function. To generate sequence statistics for raw-reads or clean-reads fastq files:

```Bash
yap stats -w [read-dir]
```

For assembly files in fasta format, use the option `--format` or `-f:

```Bash
yap stats -w [assembly-fasta-dir] --format fasta
```

## State of Code

All implemented features are working as expected. Please, expect significant code changes as the development of the program is still at the early stage.
