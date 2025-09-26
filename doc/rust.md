## impl trait 和 dyn trait 的区别
- impl trait 是在编译时确定的，而 dyn trait 是在运行时确定的
- impl trait 只能返回单一错误类型，而 dyn trait 可以返回多种错误类型
- impl trait 栈分配，而 dyn trait 堆分配
