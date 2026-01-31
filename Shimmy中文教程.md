# Shimmy Chinese Tutorial

## 1. Shimmy 是什么

Shimmy 是一个**单二进制文件**的本地 LLM 推理服务器，为 GGUF 模型提供 **100% OpenAI 兼容的端点**。将现有的 AI 工具指向 Shimmy，它们就能本地、私密、免费地工作。

### 核心特性
- **单二进制部署**: 小于 5MB，无需 Python 依赖
- **OpenAI API 兼容**: 完美兼容所有 OpenAI SDK 和工具
- **零配置**: 自动发现模型，自动分配端口
- **GPU 加速**: 支持 CUDA、Vulkan、OpenCL、MLX
- **开源免费**: MIT 许可证，永久免费

### 与其他工具对比

| 工具 | 二进制大小 | 启动时间 | OpenAI API |
|------|-----------|----------|------------|
| **Shimmy** | **4.8MB** | **<100ms** | **100%** |
| Ollama | 680MB | 5-10s | 部分兼容 |
| llama.cpp | 89MB | 1-2s | 需 llama-server |

## 2. 核心原理

### 架构设计
```
用户请求 → Shimmy HTTP Server → llama.cpp 引擎 → GGUF 模型
                              ↓
                      OpenAI API 格式响应
```

### OpenAI 兼容性
Shimmy 实现以下 OpenAI 端点:
- `POST /v1/chat/completions` - 聊天补全
- `GET /v1/models` - 列出可用模型
- `GET /health` - 健康检查
- `POST /api/generate` - Shimmy 原生 API
- `GET /ws/generate` - WebSocket 流式传输

### GGUF 格式
GGUF (GPT-Generated Unified Format) 是 llama.cpp 推出的模型格式:
- 单文件部署，无需额外权重
- 支持多种量化级别 (FP16, Q4_0, Q4_1, Q5_0, Q5_1, Q8_0)
- 自动内存映射，加速加载

## 3. 安装与配置

### 方法一: 下载预编译二进制 (推荐)

```bash
# Windows x64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-windows-x86_64.exe -o shimmy.exe

# Linux x86_64
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-linux-x86_64 -o shimmy && chmod +x shimmy

# macOS ARM64 (Apple Silicon)
curl -L https://github.com/Michael-A-Kuykendall/shimmy/releases/latest/download/shimmy-macos-arm64 -o shimmy && chmod +x shimmy
```

### 方法二: 从源码编译

```bash
# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh

# 编译安装
cargo install shimmy --features huggingface

# 带 GPU 支持编译
cargo install shimmy --features huggingface,llama,llama-cuda,llama-vulkan
```

### 获取模型

Shimmy 自动从以下位置发现模型:
- Hugging Face 缓存: `~/.cache/huggingface/hub/`
- Ollama 模型: `~/.ollama/models/`
- 本地目录: `./models/`
- 环境变量: `SHIMMY_BASE_GGUF=path/to/model.gguf`

```bash
# 使用 huggingface-cli 下载模型
huggingface-cli download microsoft/Phi-3-mini-4k-instruct-gguf --local-dir ./models/

# 或手动下载并放置到 models/ 目录
```

## 4. 快速开始

### 启动服务器

```bash
# 自动分配端口 (推荐)
./shimmy serve &

# 或指定端口
./shimmy serve --bind 127.0.0.1:11435
```

### 使用 curl 测试

```bash
# 查看可用模型
./shimmy list

# 发送请求
curl -s http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "你的模型名称",
    "messages": [{"role": "user", "content": "用5个字说你好"}],
    "max_tokens": 32
  }'
```

### 使用 Python SDK

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://127.0.0.1:11435/v1",
    api_key="sk-local"  # Shimmy 忽略此参数
)

resp = client.chat.completions.create(
    model="你的模型名称",
    messages=[{"role": "user", "content": "你好"}],
    max_tokens=32
)

print(resp.choices[0].message.content)
```

### 使用 Node.js SDK

```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
  baseURL: 'http://127.0.0.1:11435/v1',
  apiKey: 'sk-local'
});

const resp = await openai.chat.completions.create({
  model: '你的模型名称',
  messages: [{ role: 'user', content: '你好' }],
  max_tokens: 32
});

console.log(resp.choices[0].message.content);
```

## 5. API 使用详解

### 聊天补全端点

**端点**: `POST /v1/chat/completions`

**请求参数**:
```json
{
  "model": "模型名称",
  "messages": [
    {"role": "system", "content": "你是一个助手"},
    {"role": "user", "content": "你好"}
  ],
  "temperature": 0.7,
  "max_tokens": 1000,
  "stream": false
}
```

**响应**:
```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1699000000,
  "model": "模型名称",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "你好！有什么我可以帮助你的吗？"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 20,
    "total_tokens": 30
  }
}
```

### 流式响应

设置 `"stream": true` 启用 Server-Sent Events:

```bash
curl -s -N http://127.0.0.1:11435/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "模型名称",
    "messages": [{"role": "user", "content": "讲个笑话"}],
    "stream": true
  }'
```

**流式输出格式**:
```
data: {"choices":[{"delta":{"content":"有"},"index":0}]}

data: {"choices":[{"delta":{"content":"一"},"index":0}]}

data: [DONE]
```

### WebSocket 接口

**端点**: `WS /ws/generate`

```javascript
const ws = new WebSocket('ws://127.0.0.1:11435/ws/generate');

// 发送请求
ws.send(JSON.stringify({
  model: '模型名称',
  prompt: '你好',
  stream: true
}));

// 接收令牌
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.done) {
    console.log('生成完成');
  } else {
    process.stdout.write(data.token);
  }
};
```

## 6. GPU 加速

### 支持的 GPU 后端

| 后端 | 平台 | 要求 |
|------|------|------|
| CUDA | Windows/Linux | NVIDIA GPU + CUDA Toolkit |
| Vulkan | Windows/Linux/跨平台 | 兼容 Vulkan 的 GPU |
| OpenCL | Windows/Linux | AMD/Intel GPU |
| MLX | macOS | Apple Silicon |

### 自动检测 (默认)

```bash
# 自动检测并使用最佳 GPU 后端
./shimmy serve --gpu-backend auto
```

### 手动指定后端

```bash
# 强制使用 CUDA
./shimmy serve --gpu-backend cuda

# 强制使用 Vulkan
./shimmy serve --gpu-backend vulkan

# 强制使用 CPU
./shimmy serve --gpu-backend cpu
```

### 检查 GPU 状态

```bash
# 查看检测到的 GPU 后端
./shimmy gpu-info

# 详细模式查看实际使用的后端
./shimmy serve --gpu-backend auto --verbose
```

## 7. 高级功能

### MoE (混合专家) CPU 卸载

在显存有限的情况下运行 70B+ 大模型:

```bash
# 启用 MoE CPU 卸载
./shimmy serve --cpu-moe --n-cpu-moe 8
```

### 模型预加载

```bash
# 预加载指定模型
./shimmy serve --preload 模型名称

# 后台预加载多个模型
./shimmy serve --preload 模型1 --preload 模型2
```

### LoRA 适配器

```bash
# 加载 LoRA 适配器
./shimmy serve --lora path/to/lora.gguf

# 指定基础模型和 LoRA
SHIMMY_BASE_GGUF=path/to/base.gguf \
SHIMMY_LORA_GGUF=path/to/lora.gguf \
./shimmy serve
```

### 响应缓存

```bash
# 启用 LRU 缓存 (20-40% 重复查询加速)
./shimmy serve --cache
```

## 8. 常见问题

### Q1: 模型加载失败
```bash
# 检查模型文件路径
ls -lh ./models/

# 验证模型格式
file ./models/model.gguf

# 使用绝对路径
./shimmy serve --model /full/path/to/model.gguf
```

### Q2: 端口被占用
```bash
# 查看占用端口的进程
lsof -i :11435

# 使用自动端口分配
./shimmy serve

# 或指定其他端口
./shimmy serve --bind 127.0.0.1:11436
```

### Q3: 内存不足
```bash
# 使用更小的量化模型
# Q8_0 > Q5_1 > Q4_1 > Q4_0 > Q3_1

# 减少上下文长度
./shimmy serve --ctx-size 2048

# 启用 CPU 模式
./shimmy serve --gpu-backend cpu
```

### Q4: 生成速度慢
```bash
# 检查 GPU 是否被使用
./shimmy gpu-info

# 减少生成长度
"max_tokens": 100  # 而不是默认的 1000+

# 使用流式输出实时看到结果
"stream": true
```

## 9. 原理深入解析

### GGUF 格式详解

GGUF 是专为推理优化的单文件格式:

**量化级别对比**:
| 量化级别 | 模型大小 | 精度损失 | 适用场景 |
|---------|---------|---------|---------|
| FP16 | 100% | 无 | 最佳质量 |
| Q8_0 | 50% | 极小 | 接近无损 |
| Q5_1 | 32% | 小 | 平衡选择 |
| Q4_1 | 26% | 较小 | 常用选择 |
| Q4_0 | 22% | 中等 | 显存有限 |
| Q3_1 | 18% | 较大 | 极致压缩 |

**量化工具**:
```bash
# 使用 llama.cpp 量化
./quantize path/to/model.gguf path/to/model-q4_0.gguf Q4_0
```

### 分词器 (Tokenizer)

Shimmy 支持多种提示模板:

- **ChatML**: OpenAI 兼容格式
- **Llama3**: Meta 的 Llama 3 格式
- **OpenChat**: 通用聊天格式

```bash
# 查看模型使用的模板
./shimmy list --verbose
```

### 内存管理

Shimmy 使用内存映射加载模型:
- 快速启动，无需完整加载到 RAM
- 按需分页加载到 GPU 显存
- 自动释放未使用显存

### 性能优化建议

1. **使用 GPU**: GPU 加速可提升 5-50 倍
2. **选择合适量化**: Q4_1 是质量和速度的良好平衡
3. **减少上下文**: 除非必要，使用较小的 ctx-size
4. **启用缓存**: 重复查询启用 --cache
5. **批量处理**: 一次发送多个请求

---

## 参考资源

- **GitHub**: https://github.com/Michael-A-Kuykendall/shimmy
- **文档**: https://github.com/Michael-A-Kuykendall/shimmy/tree/main/docs
- **问题反馈**: https://github.com/Michael-A-Kuykendall/shimmy/issues

---

*本教程由社区贡献，欢迎提交 PR 改进！*
