## 编程作业
本次实验需要实现`sys_trace`来追踪当前任务的系统调用信息，分别包括三种功能：读取地址为id的值，在地址为id的地方写入值，记录任务系统调用次数
1. 读取地址中的值：
当 `_trace_request` 为 `0` 时，调用后，首先将 `_id` 转换为 `u8` 数据类型，然后通过函数 `read_volatile` 实现读取地址为 `_id:u8` 中的值。
2. 在地址为id的地方写入值：
当 `_trace_request` 为 `1` 时，调用后，首先将 `_id` 转换为 `u8` 数据类型，然后通过函数 `write_volatile` 写入 `data` 中的最低为的一个字节（`& 0xFF` 取最低位）。
3. 记录任务系统调用次数：
在`TaskControlBlock`中记录下来每个系统调用的次数，这里定义了一个数据`pub task_count: [usize; SYSTEM_NUM]`,`SYSTEM_NUM`是系统调用值的最大值+1，并在`TaskManager`中实现记录系统调用次数的函数`record_syscall_count`和获取系统调用次数的函数`get_syscall_count`,并作封装。在执行`sys_trace`系统调用时，`_trace_request=2`，执行对应获取系统调用函数，在Trap陷入后转入系统调用时，首先记录系统调用次数+1，即可。

## 简答作业
1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
    ```shell
    [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    ```
    出现报错，可以看到访问错误的地址或者使用 S 特权级指令，会导致系统中断，并Trap陷入内核，`scause` csr寄存器中存放了中断信息，然后内核Trap处理函数根据中断信息进行处理，发现错误行为后进行杀死。

2. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:
    1. L40：刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。
    第一次执行 `__restore` 时，sp 代表了内核用户保存上下文栈的栈顶指针。
        + `__restore` 在任务处理完成内核任务后，通过`__restore`返回用户态，并恢复该程序在用户态的上下文
        + `__restore` 当操作系统首次加载并调度一个用户任务时，内核会构造一个初始`Trap Context`，并将其地址设置为`sp`，然后调用`__restore`。
    2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
        ```assembly
        ld t0, 32*8(sp)
        ld t1, 33*8(sp)
        ld t2, 2*8(sp)
        csrw sstatus, t0
        csrw sepc, t1
        csrw sscratch, t2
        ```
        + 在`__restore`过程中，我们恢复了csr寄存器中的sstatus，sepc，sscratch，他们分别表示了cpu所处哪个特权级、Trap陷入处理后返回的位置、中转寄存器。sscratch会保存用户态中栈的栈顶指针。
    3. L50-L56：为何跳过了 x2 和 x4？
        ```assembly
        ld x1, 1*8(sp)
        ld x3, 3*8(sp)
        .set n, 5
        .rept 27
        LOAD_GP %n
        .set n, n+1
        .endr
        ```
        + x2寄存器用于存储 sp 栈顶指针，我们已经通过中转寄存器sscratch保存了，不需要再单独保存了。
        + x4寄存器除非我们手动出于一些特殊用途使用它，否则一般也不会被用到，所以它存储的值一般不发生变化，所以无需保存。
    4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
        ```assembly
        csrrw sp, sscratch, sp
        ```
        + `__restore`中 sscratch 寄存器存储的是用户Trap陷入时保存的用户栈的栈顶指针，在从内核态S到用户态U的恢复过程中，我们需要还原栈指针为用户栈，所以用sscratch来做中转。
        + `__restore`中前一个sp表示的是内核态的栈顶指针，然后会存入中转寄存器中用户栈的栈顶指针，完成内核到用户态的切换。
    5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
        ```assembly
        csrw sstatus, t0
        ```
        + t0中存放了cpu处于用户态的信息，将sstatus中spp字段修改为U，系统便完成习状态的切换。
    6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
        + sp 和 sscratch 中的值做了交换，在执行后，sp中存储的值表示内核的栈顶指针，sscratch中的值表示用户的栈空间指针。
    7. 从 U 态进入 S 态是哪一条指令发生的？
        + 当用户执行 `ecall` 系统调用后，发生系统中断（Trap类中断）,然后由寄存器stvec跳转到对应的处理函数`__alltraps`保存上下文，并实现由用户态U进入内核态S，具体指令为：1修改寄存器sstatus为内核态S，并且将用户栈指针保存，将sp指向内核栈空间，保存上下文，设置call trap_handler切换处理系统调用或者系统报错。

## 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    无
2. 此外，我也参考了以下资料，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
## Optional
看法： 
    无
