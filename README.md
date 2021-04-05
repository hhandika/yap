# YAP (Yet-Another-Pipeline)

![YAP-Tests](https://github.com/hhandika/yap/workflows/YAP-Tests/badge.svg)
[![Build Status](https://www.travis-ci.com/hhandika/yap.svg?branch=main)](https://www.travis-ci.com/hhandika/yap)

YetAnotherPipeline

A pipeline for processing high-throughput sequencing data.

## Features

| Features                                      | Dependencies                               | Implementation |
| --------------------------------------------- | ------------------------------------------ | -------------- |
| _Essentials_                                  |
| Batch adapter trimming and sequence filtering | [Fastp](https://github.com/OpenGene/fastp) | Done           |
| Batch sequence assembly                       | [SPAdes](https://github.com/ablab/spades)  | Done           |
| Sequence statistics                           | None                                       | Done           |
| _Utilities_                                   |
| Sequence finder                               | None                                       | Planned        |
| Sequence file renamer                         | None                                       | Planned        |

## Installation

OS support:

1. MacOS
2. Linux
3. Windows-WSL

## State of Code

Only checker and read assembly functions are working. Everything else is still broken.
