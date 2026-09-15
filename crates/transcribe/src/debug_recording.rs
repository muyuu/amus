//! 録音チャンクをmp3として保存する（データ収集専用、`record-audio` feature 限定）
//!
//! モデル比較の検証は手元の少数サンプルだけでは決め切れなかった。実際にアモアスで
//! 使ってもらいながら書き起こしに渡した音声そのものを集め、後で複数モデルにかけ直せる
//! ようにする。配布ビルドに混ざらないよう feature で切り離してある。
//!
//! 保存するかどうかはこのクレートでは決めない（「ターン」はアプリ側 `features::voice_memo`
//! の概念で、ここは GUI に依存しない）。呼び出し側（アプリ層）が書き起こし結果からターン
//! 開始の発話・ターン中の発話だと分かったチャンクだけ [`save_chunk`] を呼ぶ想定。

use mp3lame_encoder::{Bitrate, Builder, Encoder, FlushNoGap, MonoPcm, Quality};
use std::io::Write;
use std::path::PathBuf;

use super::voice_recorder::SAMPLE_RATE;
use crate::{log_debug, log_error};

/// 保存先ディレクトリ（実行ファイルの隣の `./recorded`）。
///
/// アプリのデータディレクトリではなくここに置く。データ収集を頼む相手には実行ファイル
/// の場所だけ伝えれば済み、集めたファイルもそのまま持ち帰りやすい。
fn output_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.join("recorded");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn build_encoder() -> Option<Encoder> {
    Builder::new()?
        .with_num_channels(1)
        .ok()?
        .with_sample_rate(SAMPLE_RATE)
        .ok()?
        // 音声のみで高音質は要らない。ファイルサイズを抑えて持ち帰りやすくする。
        .with_brate(Bitrate::Kbps64)
        .ok()?
        .with_quality(Quality::Best)
        .ok()?
        .build()
        .ok()
}

/// 音声チャンクをmp3で保存する。呼び出し側が「保存する価値がある」と判断したものだけ渡すこと。
///
/// 失敗してもログに残すだけで書き起こし自体は止めない（データ収集はあくまで副作用）。
pub fn save_chunk(samples: &[f32], start_sample: usize) {
    let Some(dir) = output_dir() else {
        log_error!("record-audio", "保存先ディレクトリ(./recorded)の用意に失敗");
        return;
    };

    let Some(mut encoder) = build_encoder() else {
        log_error!("record-audio", "mp3エンコーダの初期化に失敗");
        return;
    };

    // LAME の ieee_float 入力は ±1.0 フルスケールを想定する（±32768 を想定するのは
    // 紛らわしい名前の lame_encode_buffer_float の方）。whisper 向けの samples は
    // すでに ±1.0 に正規化済みなのでそのまま渡す。
    let mut mp3 = Vec::with_capacity(mp3lame_encoder::max_required_buffer_size(samples.len()));

    let encoded = match encoder.encode(MonoPcm(samples), mp3.spare_capacity_mut()) {
        Ok(n) => n,
        Err(e) => {
            log_error!("record-audio", format!("mp3エンコードに失敗: {:?}", e));
            return;
        }
    };
    // SAFETY: encode() は書き込んだバイト数のぶんだけ mp3.spare_capacity_mut() の
    // 先頭を初期化して返す。
    unsafe { mp3.set_len(mp3.len() + encoded) };

    let flushed = match encoder.flush::<FlushNoGap>(mp3.spare_capacity_mut()) {
        Ok(n) => n,
        Err(e) => {
            log_error!("record-audio", format!("mp3のflushに失敗: {:?}", e));
            return;
        }
    };
    // SAFETY: flush() も同様に書き込んだ範囲だけを初期化する。
    unsafe { mp3.set_len(mp3.len() + flushed) };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("{timestamp}_{start_sample:010}.mp3"));

    match std::fs::File::create(&path).and_then(|mut f| f.write_all(&mp3)) {
        Ok(()) => log_debug!("record-audio", format!("録音チャンクを保存: {:?}", path)),
        Err(e) => log_error!("record-audio", format!("録音チャンクの保存に失敗: {}", e)),
    }
}
