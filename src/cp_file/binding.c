#include <node_api.h>

// 声明ffmpeg.c中的napi函数
extern napi_value ffmpeg_run(napi_env env, napi_callback_info info);

// 声明utils.c中的napi函数
extern napi_value get_video_duration(napi_env env, napi_callback_info info);
extern napi_value get_video_format_info(napi_env env, napi_callback_info info);

napi_value Init(napi_env env, napi_value exports)
{
    napi_status status;
    napi_value fn;
    
    // 创建run函数
    status = napi_create_function(env, NULL, 0, ffmpeg_run, NULL, &fn);
    if (status != napi_ok) {
        return NULL;
    }
    status = napi_set_named_property(env, exports, "run", fn);
    if (status != napi_ok) {
        return NULL;
    }
    
    // 创建getVideoDuration函数
    status = napi_create_function(env, NULL, 0, get_video_duration, NULL, &fn);
    if (status != napi_ok) {
        return NULL;
    }
    status = napi_set_named_property(env, exports, "getVideoDuration", fn);
    if (status != napi_ok) {
        return NULL;
    }
    
    // 创建getVideoFormatInfo函数
    status = napi_create_function(env, NULL, 0, get_video_format_info, NULL, &fn);
    if (status != napi_ok) {
        return NULL;
    }
    status = napi_set_named_property(env, exports, "getVideoFormatInfo", fn);
    if (status != napi_ok) {
        return NULL;
    }
    
    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
