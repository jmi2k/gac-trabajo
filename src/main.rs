use std::{borrow::Borrow, rc::Rc};

#[macro_export]
macro_rules! helper {
    (
        // Itera sobre los elementos de la colección.
        for $pattern:pat in $collection:expr,
        // Condiciones opcionales que debe cumplir cada elemento a incluir.
        $(??? $cond:expr,)*
        // Indentación opcional para las líneas generadas.
        $(>>> $level:expr,)?
        // Cadena de texto opcional a utilizar como separador.
        $(+++ $joiner:expr,)?
        // Expresión que genera cada cadena a unir.
        $map:expr $(,)?
    ) => {{
        let mut s = String::new();
        let mut stitch = false;

        for $pattern in $collection {
            // Evalúa todas las condiciones.
            $(if !$cond { continue; })*

            // Añade el separador si es necesario.
            if stitch {
                $(s.push_str($joiner);)?
            }

            // Activa el mecanismo anterior tras procesar el primer elemento.
            stitch = true;

            // Añade el texto generado al resultado final.
            s.push_str($map.as_str());
        }

        // Aplica la indentación si se ha especificado.
        $(if $level > 0 { s = indent($level, &s); })?
        s
    }};
}

fn indent(level: usize, src: &str) -> String {
    let spaces = " ".repeat(level);
    let mut out = String::with_capacity(src.len());

    for (idx, line) in src.lines().enumerate() {
        if idx > 0 {
            out.push_str(&spaces);
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}

#[derive(Debug)]
struct Pipeline {
    name: String,
    description: String,
    clock: Edge,
    stages: Vec<Rc<Stage>>,
    hazards: Vec<Hazard>,
    forwards: Vec<Forward>,
}

#[derive(Debug)]
enum Edge {
    Posedge(String),
    Negedge(String),
    Edge(String),
}

#[derive(Debug)]
struct Stage {
    name: String,
    description: String,
    stall: Option<String>,
    flush: Option<String>,
}

#[derive(Debug)]
struct Hazard {
    name: String,
    description: String,
    condition: String,
}

#[derive(Debug)]
struct Forward {
    name: String,
    description: String,
    condition: String,
    from: Rc<Stage>,
    to: Rc<Stage>,
}

fn generate_stage_report(stage: &Rc<Stage>) -> String {
    let Stage {
        name, stall, flush, ..
    } = stage.borrow();

    let _stall_ = stall.clone().unwrap_or("1'b0".into());
    let _flush_ = flush.clone().unwrap_or("1'b0".into());

    format!(r#"$display("%s {name}\x1B[0m", status({_stall_}, {_flush_}));"#)
}

fn generate_hazard_report(stage: &Hazard) -> String {
    let Hazard {
        name, condition, ..
    } = stage;

    format!(r#"$display("%s {name}\x1B[0m", hazard_mark({condition}));"#)
}

fn generate_forward_report(stage: &Forward) -> String {
    let Forward {
        name, condition, ..
    } = stage;

    format!(r#"$display("%s {name}\x1B[0m", forward_mark({condition}));"#)
}

fn generate_testbench(pipe: &Pipeline) -> String {
    let Pipeline {
        clock,
        name,
        description,
        stages,
        hazards,
        forwards,
    } = pipe;

    let _clock_ = match clock {
        Edge::Posedge(name) => format!("posedge {name}"),
        Edge::Negedge(name) => format!("negedge {name}"),
        Edge::Edge(name) => format!("edge {name}"),
    };

    let _stages_ = helper! {
        for stage in stages,
        >>> 4,
        +++ "\n",
        generate_stage_report(stage),
    };

    let _hazards_ = helper! {
        for hazard in hazards,
        >>> 4,
        +++ "\n",
        generate_hazard_report(hazard),
    };

    let _forwards_ = helper! {
        for forward in forwards,
        >>> 4,
        +++ "\n",
        generate_forward_report(forward),
    };

    format!(
        r#"function string status(input stall, flush);

    case (1'b1)
        flush:   return "\x1B[1;31m【FLUSH】 \x1B[0m";
        stall:   return "\x1B[1;33m【STALL】 \x1B[0m";
        default: return "\x1B[1;32m【ACTIVE】\x1B[0m";
    endcase

endfunction
        
function string hazard_mark(input condition);

    if (condition) return "\x1B[1;33m ⚠ ";
    else           return "\x1B[1;32m ∅ ";

endfunction

function string forward_mark(input condition);

    if (condition) return "\x1B[1;33m → ";
    else           return "\x1B[1;37m ∅ ";

endfunction

initial begin

    $display("███   {name}: {description}   ███");
    $display();
    $display();

end

always @({_clock_}) begin

	$display("───────────────────────────────────────────────────");
	$display();

    $display("= STAGES =");
    {_stages_}
    $display();
    $display("= HAZARDS =");
    {_hazards_}
    $display();
    $display("= FORWARDS =");
    {_forwards_}
    $display();

end"#
    )
}

fn main() {
    let fetch = Rc::new(Stage {
        name: "IF".into(),
        description: "Instruction Fetch".into(),
        stall: Some("stall_fetch".into()),
        flush: Some("warp".into()),
    });

    let decode = Rc::new(Stage {
        name: "ID".into(),
        description: "Instruction Decode".into(),
        stall: Some("stall_decode".into()),
        flush: Some("warp".into()),
    });

    let execute = Rc::new(Stage {
        name: "IF".into(),
        description: "Execute".into(),
        stall: Some("stall_execute".into()),
        flush: Some("warp".into()),
    });

    let writeback = Rc::new(Stage {
        name: "WB".into(),
        description: "Write-back".into(),
        stall: None,
        flush: None,
    });

    let hazards = vec![
        Hazard {
            name: "ID/EX".into(),
            description: "foo".into(),
            condition: "conflict_decode_1 || conflict_decode_2".into(),
        },
        Hazard {
            name: "EX/EX".into(),
            description: "foo".into(),
            condition: "conflict_execute_1 || conflict_execute_2".into(),
        },
    ];

    let forwards = vec![
        Forward {
            name: "ID/EX (rs1)".into(),
            description: "foo".into(),
            condition: "conflict_decode_1".into(),
            from: decode.clone(),
            to: execute.clone(),
        },
        Forward {
            name: "ID/EX (rs2)".into(),
            description: "foo".into(),
            condition: "conflict_decode_2".into(),
            from: decode.clone(),
            to: execute.clone(),
        },
        Forward {
            name: "EX/EX (rs1)".into(),
            description: "foo".into(),
            condition: "conflict_execute_1 && !cannot_forward_execute".into(),
            from: execute.clone(),
            to: execute.clone(),
        },
        Forward {
            name: "EX/EX (rs2)".into(),
            description: "foo".into(),
            condition: "conflict_execute_2 && !cannot_forward_execute".into(),
            from: execute.clone(),
            to: execute.clone(),
        },
    ];

    let pipe = Pipeline {
        name: "RISCV".into(),
        description: "Custom RISC-V (RV32I) CPU".into(),
        clock: Edge::Posedge("clock".into()),
        stages: vec![fetch, decode, execute, writeback],
        hazards,
        forwards,
    };

    let src = generate_testbench(&pipe);
    println!("{}", src);
}
