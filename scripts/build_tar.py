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

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)

class TarExtractor:
    def __init__(self, source_file: str, output_file: str, compression_level: int = 3):
        self.source_file = source_file
        self.output_file = output_file
        self.compression_level = compression_level
        self.so_files = [
            'cangjie/tools/lib/libcjlint.so',
            'cangjie/tools/lib/libcangjie-lsp.so',
            'cangjie/runtime/lib/linux_x86_64_llvm/libsecurec.so'
        ]
        self.other_files = [
            'cangjie/tools/bin/cjlint',
            'cangjie/tools/bin/cjfmt',
            'cangjie/runtime/lib/linux_x86_64_llvm/libcangjie-runtime.so'
        ]
        self.config_dir = 'cangjie/tools/config'
        self.modules_dir = 'cangjie/modules/linux_x86_64_llvm'
        self.runtime_lib_dir = 'cangjie/runtime/lib/linux_x86_64_llvm'
    
    def _count_files_to_process(self, tar_file: tarfile.TarFile) -> int:
        count = 0
        for file_path in self.so_files:
            try:
                tar_file.getmember(file_path)
                count += 1
            except KeyError:
                pass
                
        for file_path in self.other_files:
            try:
                tar_file.getmember(file_path)
                count += 1
            except KeyError:
                pass
        
        count += len([m for m in tar_file.getmembers() 
                     if m.name.startswith(self.config_dir) and not m.isdir()])
        
        count += len([m for m in tar_file.getmembers() 
                     if m.name.startswith(self.modules_dir) and 
                     m.name.endswith('.cjo') and not m.isdir()])
        
        count += len([m for m in tar_file.getmembers() 
                     if m.name.startswith(self.runtime_lib_dir) and 
                     m.name.endswith('.so') and not m.isdir()])
        
        return count
    
    def _add_file_to_tar(self, in_tar: tarfile.TarFile, out_tar: tarfile.TarFile, 
                         src_path: str, dest_path: str) -> bool:
        try:
            member = in_tar.getmember(src_path)
            f = in_tar.extractfile(member)
            if f is not None:
                content = f.read()
                f.close()
                
                new_info = tarfile.TarInfo(dest_path)
                new_info.size = len(content)
                new_info.mode = member.mode
                new_info.uid = member.uid
                new_info.gid = member.gid
                new_info.uname = member.uname
                new_info.gname = member.gname
                new_info.mtime = member.mtime
                
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
        if original_path.startswith('cangjie/'):
            return original_path[len('cangjie/'):]
        return original_path
    
    def _compress_with_zstd(self, tar_path: str, output_path: str):
        try:
            logger.info(f"使用zstandard库压缩{tar_path}为{output_path}...")
            with open(tar_path, 'rb') as f_in:
                with open(output_path, 'wb') as f_out:
                    compressor = zstd.ZstdCompressor(level=self.compression_level, threads=-1)
                    compressor.copy_stream(f_in, f_out)
            
            original_size = os.path.getsize(tar_path)
            compressed_size = os.path.getsize(output_path)
            ratio = compressed_size / original_size * 100 if original_size > 0 else 0
            logger.info(f"压缩完成: {original_size:,} bytes -> {compressed_size:,} bytes ({ratio:.2f}%)")
            
        except Exception as e:
            logger.error(f"压缩过程中出错: {str(e)}")
            raise
    
    def process(self):
        temp_tar_file = None
        try:
            temp_fd, temp_tar_file = tempfile.mkstemp(suffix='.tar')
            os.close(temp_fd)
            
            with tarfile.open(self.source_file, 'r:gz') as in_tar:
                total_files = self._count_files_to_process(in_tar)
                logger.info(f"将处理 {total_files} 个文件")
                
                with tqdm(total=total_files, desc="处理文件", unit="文件") as pbar:
                    with tarfile.open(temp_tar_file, 'w') as out_tar:
                        for file_path in self.so_files:
                            dest_path = os.path.basename(file_path)
                            if self._add_file_to_tar(in_tar, out_tar, file_path, dest_path):
                                pbar.update(1)
                        
                        for file_path in self.other_files:
                            dest_path = self._get_trimmed_path(file_path)
                            if self._add_file_to_tar(in_tar, out_tar, file_path, dest_path):
                                pbar.update(1)
                        
                        config_files = [m for m in in_tar.getmembers() 
                                      if m.name.startswith(self.config_dir) and not m.isdir()]
                        
                        for member in config_files:
                            dest_path = self._get_trimmed_path(member.name)
                            
                            if self._add_file_to_tar(in_tar, out_tar, member.name, dest_path):
                                pbar.update(1)
                        
                        cjo_files = [m for m in in_tar.getmembers() 
                                   if m.name.startswith(self.modules_dir) and 
                                   m.name.endswith('.cjo') and not m.isdir()]
                        
                        for member in cjo_files:
                            dest_path = self._get_trimmed_path(member.name)
                            
                            if self._add_file_to_tar(in_tar, out_tar, member.name, dest_path):
                                pbar.update(1)
                        
                        runtime_so_files = [m for m in in_tar.getmembers() 
                                          if m.name.startswith(self.runtime_lib_dir) and 
                                          m.name.endswith('.so') and not m.isdir()]
                        
                        for member in runtime_so_files:
                            dest_path = self._get_trimmed_path(member.name)
                            
                            if self._add_file_to_tar(in_tar, out_tar, member.name, dest_path):
                                pbar.update(1)
            
            self._compress_with_zstd(temp_tar_file, self.output_file)
            logger.info(f"成功创建新的tar.zst文件: {self.output_file}")
            
        except Exception as e:
            logger.error(f"处理文件时发生错误: {str(e)}")
            raise
        finally:
            if temp_tar_file and os.path.exists(temp_tar_file):
                try:
                    os.remove(temp_tar_file)
                    logger.debug(f"已删除临时文件: {temp_tar_file}")
                except Exception as e:
                    logger.warning(f"清理临时文件时出错: {str(e)}")

def parse_arguments():
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
    args = parse_arguments()
    
    if args.verbose:
        logger.setLevel(logging.DEBUG)
    elif args.quiet:
        logger.setLevel(logging.ERROR)
    
    try:
        if not os.path.exists(args.source_file):
            logger.error(f"源文件不存在: {args.source_file}")
            return 1
            
        output_dir = os.path.dirname(args.output)
        if output_dir and not os.path.exists(output_dir):
            os.makedirs(output_dir)
            logger.info(f"创建输出目录: {output_dir}")
        
        if args.level < 1 or args.level > 22:
            logger.warning(f"压缩级别 {args.level} 超出范围 (1-22)，将使用默认级别 3")
            args.level = 3
        
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
            import traceback
            logger.error(traceback.format_exc())
        return 1

if __name__ == '__main__':
    sys.exit(main())