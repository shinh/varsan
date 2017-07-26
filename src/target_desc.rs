pub struct Target {
    pub user_size: usize,
    pub gp_size: usize,
    pub gp_off: isize,
    pub gp_names: Vec<String>,
    pub ip_index: usize,
    pub sp_index: usize,
    pub bp_index: usize,
}

pub fn get_target() -> Target {
    let gp_names: Vec<String> = [
        "r15",
        "r14",
        "r13",
        "r12",
        "rbp",
        "rbx",
        "r11",
        "r10",
        "r9",
        "r8",
        "rax",
        "rcx",
        "rdx",
        "rsi",
        "rdi",
        "orig_rax",
        "rip",
        "cs",
        "eflags",
        "rsp",
        "ss",
        "fs_base",
        "gs_base",
        "ds",
        "es",
        "fs",
        "gs",
    ].iter().map(|s|s.to_string()).collect();

    // #include <stddef.h>
    // #include <stdio.h>
    // #include <stdlib.h>
    // #include <sys/user.h>
    //
    // int main() {
    //   printf("user_size: %zu,\n", sizeof(struct user));
    //   printf("gp_size: %zu,\n", sizeof(void*));
    //   printf("gp_off: %zu,\n", offsetof(struct user, regs));
    // #if 0
    //   printf("fp_size: %zu,\n", sizeof(double));
    //   printf("fp_off: %zu,\n", offsetof(struct user, i387));
    // #endif
    //   printf("ip_index: %zu,\n",
    //          offsetof(struct user_regs_struct, rip) / sizeof(void*));
    //   printf("sp_index: %zu,\n",
    //          offsetof(struct user_regs_struct, rsp) / sizeof(void*));
    //   printf("bp_index: %zu,\n",
    //          offsetof(struct user_regs_struct, rbp) / sizeof(void*));
    // }
    Target {
        user_size: 912,
        gp_size: 8,
        gp_off: 0,
        gp_names: gp_names,
        ip_index: 16,
        sp_index: 19,
        bp_index: 4,
    }
}
