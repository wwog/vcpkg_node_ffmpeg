const ffmpeg = require('./build/Release/ffmpeg_node.node');
const fs = require('fs');
const path = require('path');

// 测试文件路径
const testVideoFile = 'input.mp4';

// 检查文件是否存在
if (!fs.existsSync(testVideoFile)) {
    console.error(`错误: 测试文件 ${testVideoFile} 不存在`);
    process.exit(1);
}

console.log('='.repeat(60));
console.log('开始测试视频工具函数');
console.log('='.repeat(60));
console.log(`测试文件: ${testVideoFile}\n`);

// 测试1: 获取视频时长
console.log('1. 测试 getVideoDuration:');
try {
    const duration = ffmpeg.getVideoDuration(testVideoFile);
    console.log(`   视频时长: ${duration.toFixed(2)} 秒 (${(duration / 60).toFixed(2)} 分钟)`);
} catch (error) {
    console.error(`   错误: ${error.message}`);
}
console.log('');

// 测试2: 获取视频格式信息
console.log('2. 测试 getVideoFormatInfo:');
try {
    const formatInfo = ffmpeg.getVideoFormatInfo(testVideoFile);
    console.log('   视频格式信息:');
    console.log(JSON.stringify(formatInfo, null, 2));
    
    // 详细输出各个字段
    console.log('\n   详细信息:');
    if (formatInfo.format) {
        console.log(`   格式: ${formatInfo.format}`);
    }
    if (formatInfo.duration !== undefined) {
        console.log(`   时长: ${formatInfo.duration.toFixed(2)} 秒`);
    }
    if (formatInfo.bitrate) {
        const bitrateMbps = (formatInfo.bitrate / 1000000).toFixed(2);
        console.log(`   比特率: ${formatInfo.bitrate.toLocaleString()} bps (${bitrateMbps} Mbps)`);
    }
    if (formatInfo.videoCodec) {
        console.log(`   视频编码器: ${formatInfo.videoCodec}`);
    }
    if (formatInfo.audioCodec) {
        console.log(`   音频编码器: ${formatInfo.audioCodec}`);
    }
    if (formatInfo.width && formatInfo.height) {
        console.log(`   分辨率: ${formatInfo.width}x${formatInfo.height}`);
    }
    if (formatInfo.fps) {
        console.log(`   帧率: ${formatInfo.fps.toFixed(2)} fps`);
    }
    if (formatInfo.sampleRate) {
        console.log(`   音频采样率: ${formatInfo.sampleRate} Hz`);
    }
    if (formatInfo.channels) {
        console.log(`   音频声道数: ${formatInfo.channels}`);
    }
    if (formatInfo.metadata && Object.keys(formatInfo.metadata).length > 0) {
        console.log(`   元数据:`);
        for (const [key, value] of Object.entries(formatInfo.metadata)) {
            console.log(`     ${key}: ${value}`);
        }
    }
} catch (error) {
    console.error(`   错误: ${error.message}`);
}
console.log('');

console.log('='.repeat(60));
console.log('测试完成');
console.log('='.repeat(60));

// 可选: 运行原有的转换测试
console.log('\n可选: 运行视频转换测试 (将 input.mp4 转换为 m3u8)');
console.log('如需运行转换测试，请取消下面的注释\n');


const result = ffmpeg.run([
    '-i', 'input.mp4',
    '-c:v', 'libx264',
    '-c:a', 'aac',
    '-hls_time', '10',
    '-hls_list_size', '0',
    'output.m3u8'
]);
console.log('转换结果:', result);

