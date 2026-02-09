use prop_amm_executor::{AfterSwapFn, BpfExecutor, BpfProgram, NativeExecutor, SwapFn};
use prop_amm_shared::instruction::STORAGE_SIZE;
use prop_amm_shared::nano::{f64_to_nano, nano_to_f64};

enum Backend {
    Bpf(BpfExecutor),
    Native(NativeExecutor),
}

pub struct BpfAmm {
    backend: Backend,
    pub reserve_x: f64,
    pub reserve_y: f64,
    pub name: String,
    storage: Vec<u8>,
}

impl BpfAmm {
    pub fn new(program: BpfProgram, reserve_x: f64, reserve_y: f64, name: String) -> Self {
        Self {
            backend: Backend::Bpf(BpfExecutor::new(program)),
            reserve_x,
            reserve_y,
            name,
            storage: vec![0u8; STORAGE_SIZE],
        }
    }

    pub fn new_native(swap_fn: SwapFn, after_swap_fn: Option<AfterSwapFn>, reserve_x: f64, reserve_y: f64, name: String) -> Self {
        Self {
            backend: Backend::Native(NativeExecutor::new(swap_fn, after_swap_fn)),
            reserve_x,
            reserve_y,
            name,
            storage: vec![0u8; STORAGE_SIZE],
        }
    }

    #[inline]
    fn call(&mut self, side: u8, amount: u64, rx: u64, ry: u64) -> u64 {
        match &mut self.backend {
            Backend::Bpf(exec) => exec.execute(side, amount, rx, ry, &self.storage).unwrap_or(0),
            Backend::Native(exec) => exec.execute(side, amount, rx, ry, &self.storage),
        }
    }

    #[inline]
    fn call_after_swap(&mut self, side: u8, input_amount: u64, output_amount: u64, rx: u64, ry: u64) {
        match &mut self.backend {
            Backend::Bpf(exec) => {
                let _ = exec.execute_after_swap(side, input_amount, output_amount, rx, ry, &mut self.storage);
            }
            Backend::Native(exec) => {
                exec.execute_after_swap(side, input_amount, output_amount, rx, ry, &mut self.storage);
            }
        }
    }

    #[inline]
    pub fn quote_buy_x(&mut self, input_y: f64) -> f64 {
        if input_y <= 0.0 { return 0.0; }
        nano_to_f64(self.call(0, f64_to_nano(input_y), f64_to_nano(self.reserve_x), f64_to_nano(self.reserve_y)))
    }

    #[inline]
    pub fn quote_sell_x(&mut self, input_x: f64) -> f64 {
        if input_x <= 0.0 { return 0.0; }
        nano_to_f64(self.call(1, f64_to_nano(input_x), f64_to_nano(self.reserve_x), f64_to_nano(self.reserve_y)))
    }

    #[inline]
    pub fn execute_buy_x(&mut self, input_y: f64) -> f64 {
        let output_x = self.quote_buy_x(input_y);
        if output_x > 0.0 {
            self.reserve_x -= output_x;
            self.reserve_y += input_y;
            let rx = f64_to_nano(self.reserve_x);
            let ry = f64_to_nano(self.reserve_y);
            self.call_after_swap(0, f64_to_nano(input_y), f64_to_nano(output_x), rx, ry);
        }
        output_x
    }

    #[inline]
    pub fn execute_sell_x(&mut self, input_x: f64) -> f64 {
        let output_y = self.quote_sell_x(input_x);
        if output_y > 0.0 {
            self.reserve_x += input_x;
            self.reserve_y -= output_y;
            let rx = f64_to_nano(self.reserve_x);
            let ry = f64_to_nano(self.reserve_y);
            self.call_after_swap(1, f64_to_nano(input_x), f64_to_nano(output_y), rx, ry);
        }
        output_y
    }

    #[inline]
    pub fn spot_price(&self) -> f64 {
        self.reserve_y / self.reserve_x
    }

    pub fn reset(&mut self, reserve_x: f64, reserve_y: f64) {
        self.reserve_x = reserve_x;
        self.reserve_y = reserve_y;
        self.storage.fill(0);
    }
}
