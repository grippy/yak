; ModuleID = 'helloworld.ll'
source_filename = "helloworld.ll"

@fmt_string = private unnamed_addr constant [10 x i8] c"DEBUG: %s\00", align 1
@hello_str = private unnamed_addr constant [15 x i8] c"hello, world!\0A\00", align 1

define i64 @main(i64 %0, i64 %1, i64 %2) {
enter:
  call void @print_int(i64 42)
  %buffer = alloca i8, i64 100, align 1
  %tmpcall = call i32 (i8*, i8*, ...) @sprintf(i8* %buffer, i8* getelementptr inbounds ([10 x i8], [10 x i8]* @fmt_string, i32 0, i32 0), i8* getelementptr inbounds ([15 x i8], [15 x i8]* @hello_str, i32 0, i32 0))
  %tmpcall1 = call i32 (i8*, ...) @printf(i8* %buffer)
  ret i64 0
}

declare i32 @printf(i8*, ...)

declare i32 @sprintf(i8*, i8*, ...)

declare void @print_int(i64)
