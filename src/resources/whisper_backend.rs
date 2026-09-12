//! 書き起こしの実行構成
//!
//! whisper.cpp のバックエンドはビルド時に固定される。実行時に選べるのは、ビルドに
//! 含まれている GPU バックエンドを使うかどうかだけ。

use super::whisper_transcriber::WhisperModel;

/// GPU バックエンドの種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Vulkan,
    Cuda,
    Metal,
}

impl GpuBackend {
    /// ログや画面に出す名前。
    pub fn name(self) -> &'static str {
        match self {
            Self::Vulkan => "Vulkan",
            Self::Cuda => "CUDA",
            Self::Metal => "Metal",
        }
    }
}

/// このビルドに含まれる GPU バックエンド。含まれていなければ `None`。
///
/// 複数の feature を同時に立てることは想定していない。立っていた場合は 1 つだけ採る。
pub fn compiled_backend() -> Option<GpuBackend> {
    if cfg!(feature = "gpu-vulkan") {
        Some(GpuBackend::Vulkan)
    } else if cfg!(feature = "gpu-cuda") {
        Some(GpuBackend::Cuda)
    } else if cfg!(feature = "gpu-metal") {
        Some(GpuBackend::Metal)
    } else {
        None
    }
}

/// 書き起こしをどう動かすか。バックエンドと、それに見合うモデルの組。
///
/// モデルはバックエンドと一緒に決める。処理時間の大半は音声長によらない 30 秒窓の
/// エンコードが占めており、CPU ではそれが small で頭打ちになる。GPU ならより大きい
/// モデルでも CPU + small を下回るため、精度に振れる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranscribeSetup {
    /// 使う GPU バックエンド。`None` なら CPU で動かす。
    pub gpu: Option<GpuBackend>,
    /// 使うモデル。
    pub model: WhisperModel,
}

impl TranscribeSetup {
    /// CPU で動かす構成。
    ///
    /// GPU を使わない選択のほか、GPU の初期化に失敗したときの退避先にもなる。
    pub const CPU: Self = Self {
        gpu: None,
        model: WhisperModel::SMALL,
    };

    /// このビルドの構成。
    ///
    /// GPU を使うかはビルド時に決まっており、実行時に選ぶものではない。GPU を使わせたい
    /// 環境には GPU バックエンドを含まないビルドを渡す。
    pub fn compiled() -> Self {
        Self::select(compiled_backend())
    }

    /// 何で動いているかを一目で示す表記。`Vulkan / medium` のような形。
    ///
    /// 認識がおかしいときに、GPU が効いているのか・どのモデルなのかを確かめる手がかり。
    pub fn label(&self) -> String {
        let backend = self.gpu.map(GpuBackend::name).unwrap_or("CPU");
        format!("{} / {}", backend, self.model.name())
    }

    fn select(available: Option<GpuBackend>) -> Self {
        match available {
            Some(gpu) => Self {
                gpu: Some(gpu),
                model: WhisperModel::MEDIUM,
            },
            None => Self::CPU,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gpu_build_pairs_the_backend_with_a_larger_model() {
        let setup = TranscribeSetup::select(Some(GpuBackend::Vulkan));

        assert_eq!(setup.gpu, Some(GpuBackend::Vulkan));
        assert_eq!(setup.model, WhisperModel::MEDIUM);
    }

    #[test]
    fn a_build_without_a_gpu_backend_stays_on_cpu() {
        let setup = TranscribeSetup::select(None);

        assert_eq!(setup, TranscribeSetup::CPU);
    }
}
