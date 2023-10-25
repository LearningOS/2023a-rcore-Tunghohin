# 1. 实验报告
  TaskControlBlock 添加了 start_time，和 syscall_counter，分别用于记录任务开始时间和任务使用系统调用计数，初始化任务时置 start_time 为当前时间，syscall_counter 为全 0，每次发生 syscall 增加 syscall_counter 计数。
  查询时通过 TASK_MANAGER 封装的函数查询当前任务的信息，任务用时为当前时间减去start_time。
  
# 2. 简答题
  1. 代表了trapcontext，作用之一是 trap 返回，另一是充当 task 的入口。
  2. 特殊处理了 sstatus，sepc，sscratch
      * sstatus：记录了 trap 发生前处于哪个特权态，eret 将回到此特权态。
      * sepc：记录 trap 发生时指令的地址，eret将返回到此处。
      * sscratch：作为 umode pc 和 smode pc 的中转地
  3. 暂时未被使用
  4. 原 sp 写入 sscratch，原 sscratch 写入 sp，相当于 swap
  5. eret，sstatus 指定了trap发生前来自用户态
  6. 同 4
  7. ecall

# 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    ```无```

    此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    ```https://github.com/LearningOS/rust-based-os-comp2023/blob/main/relatedinfo.md```

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。


