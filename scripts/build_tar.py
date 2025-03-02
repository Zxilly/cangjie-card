#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "tqdm",
#     "zstandard",
# ]
# ///

import argparse
import tarfile
import io
import os
import logging
import sys
import tempfile
from typing import List, Dict, Optional, Set
from tqdm import tqdm
import zstandard as zstd

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

class TarExtractor:
    """用于从tar.gz提取文件并创建新tar.zst的类"""
    
    def __init__(self, source_file: str, output_file: str, compression_level: int = 3):
        """
        初始化提取器
        
        参数:
            source_file: 源tar.gz文件的路径
            output_file: 输出tar.zst文件的路径
            compression_level: zstd压缩级别 (1-22)
        """
        self.source_file = source_file
        self.output_file = output_file
        self.compression_level = compression_level
        # 需要直接提取到根目录的.so文件
        self.so_files = [
            'cangjie/tools/lib/libcjlint.so',
            'cangjie/tools/lib/libcangjie-lsp.so',
            'cangjie/runtime/lib/linux_x86_64_llvm/libsecurec.so',
            'cangjie/runtime/lib/linux_x86_64_llvm/libcangjie-runtime.so'
        ]
        # 其他需要提取的文件（非.so文件）
        self.other_files = [
            'cangjie/tools/bin/cjlint',
            'cangjie/tools/bin/cjfmt'
        ]
        # 配置文件目录
        self.config_dir = 'cangjie/tools/config'
        # 模块目录（包含.cjo文件）
        self.modules_dir = 'cangjie/modules/linux_x86_64_llvm'
    
    def _count_files_to_process(self, tar_file: tarfile.TarFile) -> int:
        """
        计算需要处理的文件总数用于进度条显示
        
        参数:
            tar_file: 已打开的tar文件对象
            
        返回:
            需要处理的文件总数
        """
        count = 0
        # 检查.so文件
        for file_path in self.so_files:
            try:
                tar_file.getmember(file_path)
                count += 1
            except KeyError:
                pass
                
        # 检查其他文件
        for file_path in self.other_files:
            try:
                tar_file.getmember(file_path)
                count += 1
            except KeyError:
                pass
        
        # 检查配置文件
        count += len([m for m in tar_file.getmembers() 
                     if m.name.startswith(self.config_dir) and not m.isdir()])
        
        # 检查.cjo文件
        count += len([m for m in tar_file.getmembers() 
                     if m.name.startswith(self.modules_dir) and 
                     m.name.endswith('.cjo') and not m.isdir()])
        
        return count
    
    def _add_file_to_tar(self, 
                         in_tar: tarfile.TarFile, 
                         out_tar: tarfile.TarFile, 
                         src_path: str, 
                         dest_path: str) -> bool:
        """
        从源tar添加单个文件到新的tar
        
        参数:
            in_tar: 源tar文件对象
            out_tar: 目标tar文件对象
            src_path: 源文件在源tar中的路径
            dest_path: 目标文件在新tar中的路径
            
        返回:
            是否成功添加文件
        """
        try:
            # 从源文件中提取成员
            member = in_tar.getmember(src_path)
            # 从tar文件中读取文件内容到内存
            f = in_tar.extractfile(member)
            if f is not None:
                content = f.read()
                f.close()
                
                # 创建一个新的TarInfo对象
                new_info = tarfile.TarInfo(dest_path)
                new_info.size = len(content)
                new_info.mode = member.mode
                # 保留原始文件的用户/组信息
                new_info.uid = member.uid
                new_info.gid = member.gid
                new_info.uname = member.uname
                new_info.gname = member.gname
                # 保留时间戳
                new_info.mtime = member.mtime
                
                # 将文件内容添加到新的tar文件
                out_tar.addfile(new_info, io.BytesIO(content))
                logger.debug(f"添加文件: {src_path} -> {dest_path}")
                return True
            return False
        except KeyError:
            logger.warning(f"警告: 在源文件中未找到 {src_path}")
            return False
        except Exception as e:
            logger.error(f"处理文件 {src_path} 时出错: {str(e)}")
            return False
    
    def _get_trimmed_path(self, original_path: str) -> str:
        """
        去除路径中的cangjie前缀
        
        参数:
            original_path: 原始路径
            
        返回:
            去除cangjie前缀后的路径
        """
        if original_path.startswith('cangjie/'):
            return original_path[len('cangjie/'):]
        return original_path
    
    def _compress_with_zstd(self, tar_path: str, output_path: str):
        """
        使用zstandard库压缩tar文件
        
        参数:
            tar_path: 输入tar文件路径
            output_path: 输出的zst压缩文件路径
        """
        try:
            logger.info(f"使用zstandard库压缩{tar_path}为{output_path}...")
            # 创建一个默认大小的压缩缓冲区
            with open(tar_path, 'rb') as f_in:
                with open(output_path, 'wb') as f_out:
                    # 创建一个压缩器对象
                    compressor = zstd.ZstdCompressor(level=self.compression_level)
                    # 使用copy_stream将输入流的内容压缩后写入输出流
                    compressor.copy_stream(f_in, f_out)
            
            # 压缩后报告文件大小变化
            original_size = os.path.getsize(tar_path)
            compressed_size = os.path.getsize(output_path)
            ratio = compressed_size / original_size * 100 if original_size > 0 else 0
            logger.info(f"压缩完成: {original_size:,} bytes -> {compressed_size:,} bytes ({ratio:.2f}%)")
            
        except Exception as e:
            logger.error(f"压缩过程中出错: {str(e)}")
            raise
    
    def process(self):
        """执行文件提取和新tar创建的主要逻辑"""
        temp_tar_file = None
        try:
            # 创建临时文件用于tar
            temp_fd, temp_tar_file = tempfile.mkstemp(suffix='.tar')
            os.close(temp_fd)
            
            # 打开源tar.gz文件
            with tarfile.open(self.source_file, 'r:gz') as in_tar:
                # 计算总文件数用于进度条
                total_files = self._count_files_to_process(in_tar)
                logger.info(f"将处理 {total_files} 个文件")
                
                # 使用进度条
                with tqdm(total=total_files, desc="处理文件", unit="文件") as pbar:
                    # 创建一个输出的tar文件
                    with tarfile.open(temp_tar_file, 'w') as out_tar:
                        # 处理.so文件（放在根目录）
                        for file_path in self.so_files:
                            dest_path = os.path.basename(file_path)
                            if self._add_file_to_tar(in_tar, out_tar, file_path, dest_path):
                                pbar.update(1)
                        
                        # 处理其他非.so文件（保持结构但去除cangjie前缀）
                        for file_path in self.other_files:
                            dest_path = self._get_trimmed_path(file_path)
                            if self._add_file_to_tar(in_tar, out_tar, file_path, dest_path):
                                pbar.update(1)
                        
                        # 处理配置文件目录（保持结构但去除cangjie前缀）
                        config_files = [m for m in in_tar.getmembers() 
                                      if m.name.startswith(self.config_dir) and not m.isdir()]
                        
                        for member in config_files:
                            # 去除cangjie前缀保持其余路径结构
                            dest_path = self._get_trimmed_path(member.name)
                            
                            if self._add_file_to_tar(in_tar, out_tar, member.name, dest_path):
                                pbar.update(1)
                        
                        # 处理模块目录中的.cjo文件（保持结构但去除cangjie前缀）
                        cjo_files = [m for m in in_tar.getmembers() 
                                   if m.name.startswith(self.modules_dir) and 
                                   m.name.endswith('.cjo') and not m.isdir()]
                        
                        for member in cjo_files:
                            # 去除cangjie前缀保持其余路径结构
                            dest_path = self._get_trimmed_path(member.name)
                            
                            if self._add_file_to_tar(in_tar, out_tar, member.name, dest_path):
                                pbar.update(1)
            
            # 使用zstd压缩tar文件
            self._compress_with_zstd(temp_tar_file, self.output_file)
            logger.info(f"成功创建新的tar.zst文件: {self.output_file}")
            
        except Exception as e:
            logger.error(f"处理文件时发生错误: {str(e)}")
            raise
        finally:
            # 清理临时文件
            if temp_tar_file and os.path.exists(temp_tar_file):
                try:
                    os.remove(temp_tar_file)
                    logger.debug(f"已删除临时文件: {temp_tar_file}")
                except Exception as e:
                    logger.warning(f"清理临时文件时出错: {str(e)}")

def parse_arguments():
    """解析命令行参数"""
    parser = argparse.ArgumentParser(
        description='从tar.gz文件中提取特定文件并创建新的tar.zst文件。',
        formatter_class=argparse.ArgumentDefaultsHelpFormatter
    )
    parser.add_argument('source_file', 
                      help='源tar.gz文件')
    parser.add_argument('-o', '--output', 
                      default='output.tar.zst', 
                      help='输出tar.zst文件')
    parser.add_argument('-v', '--verbose', 
                      action='store_true', 
                      help='显示详细日志信息')
    parser.add_argument('-q', '--quiet', 
                      action='store_true', 
                      help='只显示错误信息')
    parser.add_argument('-l', '--level', 
                      type=int, default=19,
                      help='zstd压缩级别(1-22)')
    
    return parser.parse_args()

def main():
    """主函数"""
    args = parse_arguments()
    
    # 设置日志级别
    if args.verbose:
        logger.setLevel(logging.DEBUG)
    elif args.quiet:
        logger.setLevel(logging.ERROR)
    
    try:
        # 验证源文件存在
        if not os.path.exists(args.source_file):
            logger.error(f"源文件不存在: {args.source_file}")
            return 1
            
        # 确保输出目录存在
        output_dir = os.path.dirname(args.output)
        if output_dir and not os.path.exists(output_dir):
            os.makedirs(output_dir)
            logger.info(f"创建输出目录: {output_dir}")
        
        # 验证压缩级别
        if args.level < 1 or args.level > 22:
            logger.warning(f"压缩级别 {args.level} 超出范围 (1-22)，将使用默认级别 3")
            args.level = 3
        
        # 创建并运行提取器
        extractor = TarExtractor(args.source_file, args.output, args.level)
        extractor.process()
        return 0
        
    except ModuleNotFoundError:
        logger.error("未找到zstandard库。请安装它: pip install zstandard")
        return 1
    except KeyboardInterrupt:
        logger.info("操作被用户中断")
        return 130
    except Exception as e:
        logger.error(f"执行过程中出错: {str(e)}")
        if args.verbose:
            # 在详细模式下显示完整的堆栈跟踪
            import traceback
            logger.error(traceback.format_exc())
        return 1

if __name__ == '__main__':
    sys.exit(main())