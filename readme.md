Due to my limited skills, I tried to create a jxl binding that can work with static linking, but I was unable to do so, so I am posting a crate that works with dynamic linking.

These files are required to operate.
brotlicommon.dll
brotlidec.dll
brotlienc.dll
jxl.dll
jxl_cms.dll
jxl_jni.dll
jxl_threads.dll

Tested on Windows.
How it works is written in the tests folder.

jxl's multithreading functionality is not implemented.