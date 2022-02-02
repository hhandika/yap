use num_format::{Locale, ToFormattedString};
use std::fs::File;
use std::io::{LineWriter, Result, Write};

use crate::stats::sequence::{FastaStats, FastqStats};

pub fn write_fastq(results: &mut [FastqStats], iscsv: bool) {
    results.sort_by(|a, b| a.seqname.cmp(&b.seqname));

    log::info!("\n\x1b[1mResults:\x1b[0m");
    results.iter().for_each(|recs| {
        write_fastq_console(&recs);
    });
    log::info!("Total files: {}", results.len());

    if iscsv {
        write_fastq_csv(results);
    }
}

pub fn write_fasta(stats: &mut [FastaStats], iscsv: bool) {
    stats.sort_by(|a, b| a.seqname.cmp(&b.seqname));

    log::info!("\n\x1b[1mResults:\x1b[0m");
    stats.iter().for_each(|recs| {
        write_fasta_console(&recs);
    });
    log::info!("Total files: {}", stats.len());
    if iscsv {
        write_fasta_csv(stats);
    }
}

fn write_fasta_console(contigs: &FastaStats) {
    log::info!("\x1b[0;32mFile {:?}\x1b[0m", contigs.seqname);

    log::info!(
        "No. of contigs\t\t: {}",
        contigs.contig_counts.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Total GC count\t\t: {}",
        contigs.total_gc.to_formatted_string(&Locale::en)
    );

    log::info!("GC-content\t\t: {:.2}", &contigs.gc_content);

    log::info!(
        "Total N count\t\t: {}",
        &contigs.total_n.to_formatted_string(&Locale::en)
    );

    log::info!("N-content\t\t: {:.4}", &contigs.n_content);

    log::info!(
        "Sequence length\t\t: {} bp\n",
        contigs.total_bp.to_formatted_string(&Locale::en)
    );
    //---------------------------
    log::info!("\x1b[0;34mContigs:\x1b[0m");
    log::info!(
        "Min\t\t\t: {} bp",
        contigs.min.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Max\t\t\t: {} bp",
        &contigs.max.to_formatted_string(&Locale::en)
    );

    log::info!("Mean\t\t\t: {:.2} bp", contigs.mean);
    log::info!("Median\t\t\t: {:.2} bp", &contigs.median);
    log::info!("Stdev\t\t\t: {:.2}", &contigs.sd);

    log::info!(
        "N50\t\t\t: {}",
        &contigs.n50.to_formatted_string(&Locale::en)
    );

    log::info!(
        "N75\t\t\t: {}",
        &contigs.n75.to_formatted_string(&Locale::en)
    );

    log::info!(
        "N90\t\t\t: {}",
        &contigs.n90.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Contigs >750 bp\t\t: {}",
        &contigs.con750.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Contigs >1000 bp\t: {}",
        &contigs.con1000.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Contigs >1500 bp\t: {}",
        &contigs.con1500.to_formatted_string(&Locale::en)
    );
    println!();
}

fn write_fastq_console(all_reads: &FastqStats) {
    log::info!("\x1b[0;32mFile {:?}\x1b[0m", &all_reads.seqname);

    log::info!(
        "No. of reads\t\t: {}",
        &all_reads.read_count.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Total GC count\t\t: {}",
        &all_reads.total_gc.to_formatted_string(&Locale::en)
    );

    log::info!("GC-content\t\t: {:.2}", &all_reads.gc_content);

    log::info!(
        "Total N count\t\t: {}",
        &all_reads.total_n.to_formatted_string(&Locale::en)
    );

    log::info!("N-content\t\t: {:.4}", &all_reads.n_content);

    log::info!(
        "Sequence length\t\t: {} bp\n",
        &all_reads.total_bp.to_formatted_string(&Locale::en)
    );
    //---------------------------
    log::info!("\x1b[0;34mReads:\x1b[0m");

    log::info!(
        "Min\t\t\t: {} bp",
        &all_reads.min_reads.to_formatted_string(&Locale::en)
    );

    log::info!(
        "Max\t\t\t: {} bp",
        &all_reads.max_reads.to_formatted_string(&Locale::en)
    );

    log::info!("Mean\t\t\t: {:.2} bp", &all_reads.mean_reads);
    log::info!("Median\t\t\t: {:.2} bp", &all_reads.median_reads);
    log::info!("Stdev\t\t\t: {:.2}\n", &all_reads.sd_reads);
    //--------------------
    log::info!("\x1b[0;34mPhred Q-Scores:\x1b[0m");
    log::info!("Mean\t\t\t: {:.2}", &all_reads.mean_qscores);
    log::info!(
        "Bases < 20\t\t: {}",
        &all_reads.sum_low_bases.to_formatted_string(&Locale::en)
    );

    log::info!("Low Q-score ratio\t: {:.2}\n", &all_reads.low_bases_ratio);
    if all_reads.total_bp != all_reads.sum_qlen {
        log::warn!(
            "\x1b[0;33mWARNING!\n\
            \x1b[3mSome bases may not have Q-score.\n\
            The Q-score length and the sequence length are not equal.\
            \x1b[0m\n"
        );
    }
}

fn write_fastq_csv(all_reads: &[FastqStats]) {
    let fname = "YAP-fastq-summary.csv";
    let output = File::create(&fname).expect("FILE EXISTS.");
    let mut line = LineWriter::new(output);
    let path = !all_reads[0].path.is_empty();

    write_fastq_header(&mut line, path).expect("Failed writing csv header.");

    all_reads.iter().for_each(|seq| {
        write_fastq_contents(seq, &mut line, path).expect("Failed writing csv content")
    });
    log::info!("The result is saved as {}", fname);
}

fn write_fasta_csv(stats: &[FastaStats]) {
    let fname = "YAP-fasta-summary.csv";
    let output = File::create(&fname).expect("FILE EXISTS.");
    let mut line = LineWriter::new(output);
    let path = !stats[0].path.is_empty();

    write_fasta_header(&mut line, path).expect("Failed writing csv header");

    stats.iter().for_each(|seq| {
        write_fasta_contents(seq, &mut line, path).expect("Failed writing csv content")
    });
    log::info!("The result is saved as {}", fname);
}

fn write_fastq_header<W: Write>(line: &mut W, path: bool) -> Result<()> {
    if path {
        write!(line, "Path,")?;
    }
    writeln!(
        line,
        "Sequence names,\
        Read counts,\
        Total sequence length,\
        GC counts,\
        GC-content,\
        N counts,\
        N-content,\
        Min read length,\
        Max read length,\
        Mean read length,\
        Median read length,\
        Stdev read length,\
        Mean q-score,\
        Low base < 20,\
        Low q-score ratio"
    )?;

    Ok(())
}

fn write_fastq_contents<W: Write>(seq: &FastqStats, line: &mut W, path: bool) -> Result<()> {
    if path {
        write!(line, "{},", seq.path).unwrap();
    }
    writeln!(
        line,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        seq.seqname,
        seq.read_count,
        seq.total_bp,
        seq.total_gc,
        seq.gc_content,
        seq.total_n,
        seq.n_content,
        seq.min_reads,
        seq.max_reads,
        seq.mean_reads,
        seq.median_reads,
        seq.sd_reads,
        seq.mean_qscores,
        seq.sum_low_bases,
        seq.low_bases_ratio,
    )?;

    Ok(())
}

fn write_fasta_header<W: Write>(line: &mut W, path: bool) -> Result<()> {
    if path {
        write!(line, "Path,").unwrap();
    }
    writeln!(
        line,
        "Sequence_names,\
        Contig_counts,\
        Total_sequence_length,\
        GC_counts,\
        GC-content,\
        N_counts,\
        N-content,\
        Min_contig_length,\
        Max_contig_length,\
        Mean_contig_length,\
        Median_contig_length,\
        Stdev_contig_length,\
        N50,\
        N75,\
        N90,\
        No_contigs_>750bp,\
        No_contigs_>1000bp,\
        No_contigs_>1500bp"
    )?;

    Ok(())
}

fn write_fasta_contents<W: Write>(seq: &FastaStats, line: &mut W, path: bool) -> Result<()> {
    if path {
        write!(line, "{},", seq.path).unwrap();
    }
    writeln!(
        line,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        seq.seqname,
        seq.contig_counts,
        seq.total_bp,
        seq.total_gc,
        seq.gc_content,
        seq.total_n,
        seq.n_content,
        seq.min,
        seq.max,
        seq.mean,
        seq.median,
        seq.sd,
        seq.n50,
        seq.n75,
        seq.n90,
        seq.con750,
        seq.con1000,
        seq.con1500,
    )?;

    Ok(())
}
