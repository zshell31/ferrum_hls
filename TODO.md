- ConstVal использовать длинную арифметику
- Добавить преобразование нод для случая:
```
/* Automatically generated by Ferrum. */
/*
pub fn top_module(r#in: Signal<TestSystem, Unsigned<4>>) -> Signal<TestSystem, Unsigned<2>> {
    r#in.map(|r#in| {
        let r#in = r#in.repack::<[Bit; 4]>();
        let enc = match r#in {
            [_, _, _, H] => u(0),
            [_, _, H, L] => u(1),
            [_, H, L, L] => u(2),
            _ => u(3),
        };
        enc.into()
    })
}
/*

module top_module
(
    // Inputs
    input wire [3:0] _in,
    // Outputs
    output wire [1:0] __tmp_31
);

    wire [1:0] __tmp_28;
    always @(*) begin
        casez ({ _in[3], _in[2], _in[1], _in[0] }) // <----- convert to _in
            4'b???1 : __tmp_28 = 2'd0;
            4'b??10 : __tmp_28 = 2'd1;
            4'b?100 : __tmp_28 = 2'd2;
            default: __tmp_28 = 2'd3;
        endcase
    end

    wire [1:0] __tmp_31;
    assign __tmp_31 = __tmp_28;

endmodule

```
- Тесты
- Доработать sim watch
- Добавить возможность записи в vcd
- README / Documentation
- Определиться с типом для ширины сигналов (сейчас используется u128, но это много)

- NetList transform:
  - extend -> splitter
  - merger -> splitter
