/*
 * utils.c - 视频工具函数
 * 提供获取视频时长、大小和格式信息等功能
 */

#include <node_api.h>
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libavutil/dict.h>
#include <libavutil/rational.h>
#include <libavutil/channel_layout.h>
#include <string.h>
#include <sys/stat.h>
#include <errno.h>

#ifdef _WIN32
#include <windows.h>
#include <io.h>
#else
#include <unistd.h>
#endif

/**
 * 获取视频时长
 * 参数: [文件路径]
 * 返回: 时长（秒，double）
 */
napi_value get_video_duration(napi_env env, napi_callback_info info)
{
    napi_status status;
    size_t argc = 1;
    napi_value argv[1];
    napi_value result;
    char filepath[1024];
    size_t filepath_len;
    AVFormatContext *fmt_ctx = NULL;
    int ret;
    double duration = 0.0;
    
    // 获取参数
    status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get callback info");
        return NULL;
    }
    
    if (argc < 1) {
        napi_throw_type_error(env, NULL, "Expected file path as argument");
        return NULL;
    }
    
    // 获取文件路径字符串
    status = napi_get_value_string_utf8(env, argv[0], filepath, sizeof(filepath), &filepath_len);
    if (status != napi_ok) {
        napi_throw_type_error(env, NULL, "Invalid file path");
        return NULL;
    }
    
    // 初始化 FFmpeg
    av_log_set_level(AV_LOG_QUIET);
    
    // 打开输入文件
    ret = avformat_open_input(&fmt_ctx, filepath, NULL, NULL);
    if (ret < 0) {
        char error_msg[256];
        snprintf(error_msg, sizeof(error_msg), "Could not open file: %s", filepath);
        napi_throw_error(env, NULL, error_msg);
        return NULL;
    }
    
    // 查找流信息
    ret = avformat_find_stream_info(fmt_ctx, NULL);
    if (ret < 0) {
        avformat_close_input(&fmt_ctx);
        napi_throw_error(env, NULL, "Could not find stream information");
        return NULL;
    }
    
    // 获取时长（秒）
    if (fmt_ctx->duration != AV_NOPTS_VALUE) {
        duration = (double)fmt_ctx->duration / AV_TIME_BASE;
    }
    
    // 清理资源
    avformat_close_input(&fmt_ctx);
    
    // 返回结果
    status = napi_create_double(env, duration, &result);
    if (status != napi_ok) {
        return NULL;
    }
    
    return result;
}

/**
 * 获取视频格式信息（元数据）
 * 参数: [文件路径]
 * 返回: 包含格式信息的对象
 */
napi_value get_video_format_info(napi_env env, napi_callback_info info)
{
    napi_status status;
    size_t argc = 1;
    napi_value argv[1];
    napi_value result;
    napi_value format_name, duration, bitrate, video_codec, audio_codec;
    napi_value width, height, fps, metadata_obj;
    char filepath[1024];
    size_t filepath_len;
    AVFormatContext *fmt_ctx = NULL;
    AVCodecParameters *video_codecpar = NULL;
    AVCodecParameters *audio_codecpar = NULL;
    int ret;
    int video_stream_idx = -1;
    int audio_stream_idx = -1;
    
    // 获取参数
    status = napi_get_cb_info(env, info, &argc, argv, NULL, NULL);
    if (status != napi_ok) {
        napi_throw_error(env, NULL, "Failed to get callback info");
        return NULL;
    }
    
    if (argc < 1) {
        napi_throw_type_error(env, NULL, "Expected file path as argument");
        return NULL;
    }
    
    // 获取文件路径字符串
    status = napi_get_value_string_utf8(env, argv[0], filepath, sizeof(filepath), &filepath_len);
    if (status != napi_ok) {
        napi_throw_type_error(env, NULL, "Invalid file path");
        return NULL;
    }
    
    // 初始化 FFmpeg
    av_log_set_level(AV_LOG_QUIET);
    
    // 打开输入文件
    ret = avformat_open_input(&fmt_ctx, filepath, NULL, NULL);
    if (ret < 0) {
        char error_msg[256];
        snprintf(error_msg, sizeof(error_msg), "Could not open file: %s", filepath);
        napi_throw_error(env, NULL, error_msg);
        return NULL;
    }
    
    // 查找流信息
    ret = avformat_find_stream_info(fmt_ctx, NULL);
    if (ret < 0) {
        avformat_close_input(&fmt_ctx);
        napi_throw_error(env, NULL, "Could not find stream information");
        return NULL;
    }
    
    // 查找视频流和音频流
    for (unsigned int i = 0; i < fmt_ctx->nb_streams; i++) {
        if (fmt_ctx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO && video_stream_idx < 0) {
            video_stream_idx = i;
        } else if (fmt_ctx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_AUDIO && audio_stream_idx < 0) {
            audio_stream_idx = i;
        }
    }
    
    // 创建结果对象
    status = napi_create_object(env, &result);
    if (status != napi_ok) {
        avformat_close_input(&fmt_ctx);
        return NULL;
    }
    
    // 设置格式名称
    if (fmt_ctx->iformat && fmt_ctx->iformat->name) {
        status = napi_create_string_utf8(env, fmt_ctx->iformat->name, NAPI_AUTO_LENGTH, &format_name);
        if (status == napi_ok) {
            napi_set_named_property(env, result, "format", format_name);
        }
    }
    
    // 设置时长
    if (fmt_ctx->duration != AV_NOPTS_VALUE) {
        double duration_sec = (double)fmt_ctx->duration / AV_TIME_BASE;
        status = napi_create_double(env, duration_sec, &duration);
        if (status == napi_ok) {
            napi_set_named_property(env, result, "duration", duration);
        }
    }
    
    // 设置比特率
    if (fmt_ctx->bit_rate > 0) {
        status = napi_create_int64(env, fmt_ctx->bit_rate, &bitrate);
        if (status == napi_ok) {
            napi_set_named_property(env, result, "bitrate", bitrate);
        }
    }
    
    // 设置视频编码器
    if (video_stream_idx >= 0) {
        video_codecpar = fmt_ctx->streams[video_stream_idx]->codecpar;
        const AVCodec *codec = avcodec_find_decoder(video_codecpar->codec_id);
        if (codec && codec->name) {
            status = napi_create_string_utf8(env, codec->name, NAPI_AUTO_LENGTH, &video_codec);
            if (status == napi_ok) {
                napi_set_named_property(env, result, "videoCodec", video_codec);
            }
        }
        
        // 设置视频宽度和高度
        if (video_codecpar->width > 0) {
            status = napi_create_int32(env, video_codecpar->width, &width);
            if (status == napi_ok) {
                napi_set_named_property(env, result, "width", width);
            }
        }
        
        if (video_codecpar->height > 0) {
            status = napi_create_int32(env, video_codecpar->height, &height);
            if (status == napi_ok) {
                napi_set_named_property(env, result, "height", height);
            }
        }
        
        // 设置帧率
        AVRational fps_rational = fmt_ctx->streams[video_stream_idx]->r_frame_rate;
        if (fps_rational.num > 0 && fps_rational.den > 0) {
            double fps_value = av_q2d(fps_rational);
            status = napi_create_double(env, fps_value, &fps);
            if (status == napi_ok) {
                napi_set_named_property(env, result, "fps", fps);
            }
        }
    }
    
    // 设置音频编码器
    if (audio_stream_idx >= 0) {
        audio_codecpar = fmt_ctx->streams[audio_stream_idx]->codecpar;
        const AVCodec *codec = avcodec_find_decoder(audio_codecpar->codec_id);
        if (codec && codec->name) {
            status = napi_create_string_utf8(env, codec->name, NAPI_AUTO_LENGTH, &audio_codec);
            if (status == napi_ok) {
                napi_set_named_property(env, result, "audioCodec", audio_codec);
            }
        }
        
        // 设置音频采样率
        if (audio_codecpar->sample_rate > 0) {
            napi_value sample_rate;
            status = napi_create_int32(env, audio_codecpar->sample_rate, &sample_rate);
            if (status == napi_ok) {
                napi_set_named_property(env, result, "sampleRate", sample_rate);
            }
        }
        
        // 设置音频声道数
        {
            int nb_channels = 0;
            // 尝试使用新的 ch_layout API (FFmpeg 5.0+)
            if (audio_codecpar->ch_layout.nb_channels > 0) {
                nb_channels = audio_codecpar->ch_layout.nb_channels;
            }
            // 如果新 API 不可用，尝试使用旧的 channels 字段
            #if LIBAVCODEC_VERSION_MAJOR < 59
            if (nb_channels == 0 && audio_codecpar->channels > 0) {
                nb_channels = audio_codecpar->channels;
            }
            #endif
            
            if (nb_channels > 0) {
                napi_value channels;
                status = napi_create_int32(env, nb_channels, &channels);
                if (status == napi_ok) {
                    napi_set_named_property(env, result, "channels", channels);
                }
            }
        }
    }
    
    // 设置元数据
    if (fmt_ctx->metadata) {
        status = napi_create_object(env, &metadata_obj);
        if (status == napi_ok) {
            AVDictionaryEntry *tag = NULL;
            while ((tag = av_dict_get(fmt_ctx->metadata, "", tag, AV_DICT_IGNORE_SUFFIX))) {
                napi_value key, value;
                status = napi_create_string_utf8(env, tag->key, NAPI_AUTO_LENGTH, &key);
                if (status == napi_ok) {
                    status = napi_create_string_utf8(env, tag->value, NAPI_AUTO_LENGTH, &value);
                    if (status == napi_ok) {
                        napi_set_property(env, metadata_obj, key, value);
                    }
                }
            }
            napi_set_named_property(env, result, "metadata", metadata_obj);
        }
    }
    
    // 清理资源
    avformat_close_input(&fmt_ctx);
    
    return result;
}

