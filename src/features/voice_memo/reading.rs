//! 読みの正規化と近似マッチ
//!
//! 音声認識の結果は表記が揺れる。ひらがな・カタカナ、長音符の書き方、区切り記号の
//! 有無で同じ発話が別の文字列になる。読みへ揃えた上で多少の誤りを許して照合する。
//!
//! ホットワード検知と語彙補正がこの仕組みを共有する。

/// テキスト中の、そのトリガーワードと十分近い並びの範囲。無ければ `None`。
///
/// 音声認識は語の一部を取り違えるため、完全一致では取り逃す。許容する誤りは語長に
/// 比例させ、短い語で誤爆しないようにする。
pub(super) fn match_range(text: &[char], form: &[char]) -> Option<(usize, usize)> {
    if form.is_empty() {
        return None;
    }

    let (distance, end) = closest_substring(text, form);
    if distance > form.len() / 4 {
        return None;
    }

    // 後ろ向きに同じことをすると先頭が出る
    let head: Vec<char> = text[..end].iter().rev().copied().collect();
    let reversed: Vec<char> = form.iter().rev().copied().collect();
    let (_, length) = closest_substring(&head, &reversed);

    Some((end - length, end))
}

/// パターンに最も近い部分文字列の、編集距離とその終了位置。
///
/// 先頭行を 0 で埋めることで、テキストのどの位置から照合を始めてもよいことを表す。
fn closest_substring(text: &[char], pattern: &[char]) -> (usize, usize) {
    let mut prev = vec![0usize; text.len() + 1];
    let mut cur = vec![0usize; text.len() + 1];

    for (i, &p) in pattern.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &t) in text.iter().enumerate() {
            let substitute = prev[j] + usize::from(p != t);
            cur[j + 1] = substitute.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    // 同じ距離なら手前で切る。語の後ろの発話を巻き込まないため。
    prev.iter()
        .enumerate()
        .min_by_key(|&(end, &distance)| (distance, end))
        .map(|(end, &distance)| (distance, end))
        .unwrap_or((pattern.len(), 0))
}

/// 照合用に揃えた読みと、元テキストの文字位置との対応。
pub(super) struct Reading {
    pub(super) chars: Vec<char>,
    /// `chars[i]` が元テキストのどこから始まるか
    source_start: Vec<usize>,
    /// `chars[i]` が元テキストのどこまでに対応するか（その文字の直後）
    source_end: Vec<usize>,
}

impl Reading {
    /// 読みの `start` 文字目が、元テキストのどこから始まるか。
    ///
    /// 落とした区切り記号を巻き込まないよう、その文字自身の位置を返す。
    pub(super) fn source_start(&self, start: usize) -> usize {
        self.source_start.get(start).copied().unwrap_or(0)
    }

    /// 読みの `end` 文字目までが、元テキストのどこまでに当たるか。
    pub(super) fn source_end(&self, end: usize) -> usize {
        end.checked_sub(1)
            .and_then(|last| self.source_end.get(last).copied())
            .unwrap_or(0)
    }
}

/// 照合用に読みを揃える。
///
/// 音声認識の結果は区切り記号や空白が入りうるため落とし、ひらがなはカタカナへ寄せる。
/// 長音符は直前の母音へ開く。同じ読みが「ターン」とも「たあん」とも書かれるため。
pub(super) fn read(text: &str) -> Reading {
    let mut chars = Vec::new();
    let mut source_start = Vec::new();
    let mut source_end = Vec::new();

    for (index, c) in text.chars().enumerate() {
        if c.is_whitespace() || is_separator(c) {
            continue;
        }

        let c = to_katakana(c);
        if c == 'ー' {
            // 直前に母音がなければ開きようがないので落とす
            match chars.last().copied().and_then(vowel_of) {
                Some(vowel) => chars.push(vowel),
                None => continue,
            }
        } else {
            chars.push(c);
        }
        source_start.push(index);
        source_end.push(index + 1);
    }

    Reading {
        chars,
        source_start,
        source_end,
    }
}

/// 前後の区切り記号と空白を落とす。
pub(super) fn trim_separators(text: &str) -> &str {
    text.trim_matches(|c: char| c.is_whitespace() || is_separator(c))
}

/// カタカナの母音。母音を持たない文字（「ン」「ッ」や漢字）は `None`。
fn vowel_of(c: char) -> Option<char> {
    const ROWS: [(char, &str); 5] = [
        ('ア', "アァカガサザタダナハバパマヤャラワヮ"),
        ('イ', "イィキギシジチヂニヒビピミリヰ"),
        ('ウ', "ウゥクグスズツヅヌフブプムユュルヴ"),
        ('エ', "エェケゲセゼテデネヘベペメレヱ"),
        ('オ', "オォコゴソゾトドノホボポモヨョロヲ"),
    ];

    ROWS.iter()
        .find(|(_, row)| row.contains(c))
        .map(|(vowel, _)| *vowel)
}

fn is_separator(c: char) -> bool {
    matches!(
        c,
        '、' | '。' | '，' | '．' | ',' | '.' | '!' | '?' | '！' | '？' | '・' | '-'
    )
}

/// ひらがなをカタカナへ寄せる。カタカナ・漢字・その他はそのまま。
fn to_katakana(c: char) -> char {
    match c {
        'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
        _ => c,
    }
}
