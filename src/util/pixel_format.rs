use std::slice;

use log::warn;
use rsmpeg::{ffi::{
    // AVPixelFormat_AV_PIX_FMT_ABGR, AVPixelFormat_AV_PIX_FMT_BAYER_BGGR8,
    // AVPixelFormat_AV_PIX_FMT_BAYER_GRBG16, AVPixelFormat_AV_PIX_FMT_BGR32,
    // AVPixelFormat_AV_PIX_FMT_BGR4_BYTE, AVPixelFormat_AV_PIX_FMT_BGR565BE,
    // AVPixelFormat_AV_PIX_FMT_D3D11VA_VLD, AVPixelFormat_AV_PIX_FMT_DRM_PRIME,
    // AVPixelFormat_AV_PIX_FMT_GBRAP12LE, AVPixelFormat_AV_PIX_FMT_GBRAPF32BE,
    // AVPixelFormat_AV_PIX_FMT_GBRP12BE, AVPixelFormat_AV_PIX_FMT_GRAY10BE,
    // AVPixelFormat_AV_PIX_FMT_GRAY12BE, AVPixelFormat_AV_PIX_FMT_GRAY16LE,
    // AVPixelFormat_AV_PIX_FMT_NV16, AVPixelFormat_AV_PIX_FMT_NV20LE,
    // AVPixelFormat_AV_PIX_FMT_P010BE, AVPixelFormat_AV_PIX_FMT_P210BE,
    // AVPixelFormat_AV_PIX_FMT_RGB32, AVPixelFormat_AV_PIX_FMT_RGB48,
    // AVPixelFormat_AV_PIX_FMT_RGB4_BYTE, AVPixelFormat_AV_PIX_FMT_VDPAU,
    // AVPixelFormat_AV_PIX_FMT_X2RGB10LE, AVPixelFormat_AV_PIX_FMT_XYZ12BE,
    // AVPixelFormat_AV_PIX_FMT_YA16BE, 
    AVPixelFormat_AV_PIX_FMT_YUV420P as AVPIXELFORMAT_AV_PIX_FMT_YUV420P,
    // AVPixelFormat_AV_PIX_FMT_YUV420P12, AVPixelFormat_AV_PIX_FMT_YUV420P16,
    // AVPixelFormat_AV_PIX_FMT_YUV420P9, AVPixelFormat_AV_PIX_FMT_YUV422P,
    // AVPixelFormat_AV_PIX_FMT_YUV422P14LE, AVPixelFormat_AV_PIX_FMT_YUV440P10LE,
    // AVPixelFormat_AV_PIX_FMT_YUV444P10, AVPixelFormat_AV_PIX_FMT_YUV444P16,
    // AVPixelFormat_AV_PIX_FMT_YUVA420P16, AVPixelFormat_AV_PIX_FMT_YUVA420P9,
    // AVPixelFormat_AV_PIX_FMT_YUVA420P9LE, AVPixelFormat_AV_PIX_FMT_YUVA422P16LE,
    // AVPixelFormat_AV_PIX_FMT_YUVA444P12LE, AVPixelFormat_AV_PIX_FMT_YUVA444P16LE,
    // AVPixelFormat_AV_PIX_FMT_YUVJ422P
}, avutil::AVFrame};

use crate::{global::VIDEO_SUMMARY, media::decoder::VideoFrame};

pub fn parse_video_frame(frame: &AVFrame) -> VideoFrame {
    let r = VIDEO_SUMMARY.read().unwrap();
    let summary = r.as_ref().unwrap();
    let width = frame.width as usize;
    let height = frame.height as usize;

    match frame.format {
        AVPIXELFORMAT_AV_PIX_FMT_YUV420P => {
            let y_size = width * height;
            let u_size = y_size / 4; // width/2 * height/2
            let v_size = y_size / 4; // width/2 * height/2

            let y_ptr = frame.data[0];
            let y = unsafe { slice::from_raw_parts(y_ptr, y_size) }.to_vec();

            let u_ptr = frame.data[1];
            let u = unsafe { slice::from_raw_parts(u_ptr, u_size) }.to_vec();

            let v_ptr = frame.data[2];
            let v = unsafe { slice::from_raw_parts(v_ptr, v_size) }.to_vec();

            VideoFrame {
                format: frame.format,
                data: [y, u, v, vec![], vec![], vec![], vec![], vec![]],
                width,
                height,
                pts: frame.pts,
                pts_millis: 1000 * frame.pts * summary.timebase_num as i64
                    / summary.timebase_den as i64,
            }
        }
        // AVPixelFormat_AV_PIX_FMT_ABGR => {}
        // AVPixelFormat_AV_PIX_FMT_BAYER_BGGR8 => {}
        // AVPixelFormat_AV_PIX_FMT_BAYER_GRBG16 => {}
        // AVPixelFormat_AV_PIX_FMT_BGR32 => {}
        // AVPixelFormat_AV_PIX_FMT_BGR4_BYTE => {}
        // AVPixelFormat_AV_PIX_FMT_BGR565BE => {}
        // AVPixelFormat_AV_PIX_FMT_D3D11VA_VLD => {}
        // AVPixelFormat_AV_PIX_FMT_DRM_PRIME => {}
        // AVPixelFormat_AV_PIX_FMT_GBRAP12LE => {}
        // AVPixelFormat_AV_PIX_FMT_GBRAPF32BE => {}
        // AVPixelFormat_AV_PIX_FMT_GBRP12BE => {}
        // AVPixelFormat_AV_PIX_FMT_RGB4_BYTE => {}
        // AVPixelFormat_AV_PIX_FMT_GRAY10BE => {}
        // AVPixelFormat_AV_PIX_FMT_GRAY12BE => {}
        // AVPixelFormat_AV_PIX_FMT_GRAY16LE => {}
        // AVPixelFormat_AV_PIX_FMT_NV16 => {}
        // AVPixelFormat_AV_PIX_FMT_NV20LE => {}
        // AVPixelFormat_AV_PIX_FMT_P010BE => {}
        // AVPixelFormat_AV_PIX_FMT_P210BE => {}
        // AVPixelFormat_AV_PIX_FMT_RGB32 => {}
        // AVPixelFormat_AV_PIX_FMT_RGB48 => {}
        // AVPixelFormat_AV_PIX_FMT_VDPAU => {}
        // AVPixelFormat_AV_PIX_FMT_X2RGB10LE => {}
        // AVPixelFormat_AV_PIX_FMT_XYZ12BE => {}
        // AVPixelFormat_AV_PIX_FMT_YA16BE => {}
        // AVPixelFormat_AV_PIX_FMT_YUV420P12 => {}
        // AVPixelFormat_AV_PIX_FMT_YUV420P16 => {}
        // AVPixelFormat_AV_PIX_FMT_YUV420P9 => {}
        // AVPixelFormat_AV_PIX_FMT_YUV422P => {}
        // AVPixelFormat_AV_PIX_FMT_YUV422P14LE => {}
        // AVPixelFormat_AV_PIX_FMT_YUV440P10LE => {}
        // AVPixelFormat_AV_PIX_FMT_YUV444P10 => {}
        // AVPixelFormat_AV_PIX_FMT_YUV444P16 => {}
        // AVPixelFormat_AV_PIX_FMT_YUVA420P16 => {}
        // AVPixelFormat_AV_PIX_FMT_YUVA420P9 => {}
        // AVPixelFormat_AV_PIX_FMT_YUVA420P9LE => {}
        // AVPixelFormat_AV_PIX_FMT_YUVA422P16LE => {}
        // AVPixelFormat_AV_PIX_FMT_YUVA444P12LE => {}
        // AVPixelFormat_AV_PIX_FMT_YUVA444P16LE => {}
        // AVPixelFormat_AV_PIX_FMT_YUVJ422P => {}
        _ => {
            warn!(
                "Un implemented pixel format: {}. It needs some time to finish the work.",
                frame.format
            );
            VideoFrame {
                format: frame.format,
                data: [
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                ],
                width,
                height,
                pts: frame.pts,
                pts_millis: frame.pts * 1000 * frame.time_base.num as i64
                    / frame.time_base.den as i64,
            }
        }
    }
}
