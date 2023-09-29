# note-porter
笔记导入、导出工具

## 为知笔记导出全部笔记

* [x] 登录
* [x] 获取笔记本及笔记元数据
* [x] 获取笔记详情
* [x] 笔记本及笔记内容保存到本地文件
* [ ] 非文档类型(事件)
* [ ] 加密文档
* [ ] 下载附件

## 使用

下载所有目录及笔记写入本地文件

```shell
cargo run export --output-dir /path/to/wiz --from wiz
```