use clap::Parser;
use std::num::ParseIntError;
use std::process::Command;
use std::process::Stdio;

/// 自定义数值解析函数（支持十进制、十六进制、二进制）
fn parse_number(s: &str) -> Result<i64, ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x") {
        i64::from_str_radix(hex, 16)
    } else if let Some(bin) = s.strip_prefix("0b") {
        i64::from_str_radix(bin, 2)
    } else {
        s.parse::<i64>()
    }
}

/// 表达式中的元素：数字或操作符
#[derive(Debug, Clone)] // 派生 Clone 特性
enum ExprToken {
    Number(i64),
    Operator(char),
    LeftParen,  // 左中括号 [
    RightParen, // 右中括号 ]
}

/// 解析单个表达式元素
fn parse_expression_token(input: &str) -> Result<ExprToken, String> {
    if let Ok(num) = parse_number(input) {
        Ok(ExprToken::Number(num))
    } else if "+x/".contains(input) && input.len() == 1 {
        Ok(ExprToken::Operator(input.chars().next().unwrap()))
    } else if input == "[" {
        Ok(ExprToken::LeftParen)
    } else if input == "]" {
        Ok(ExprToken::RightParen)
    } else {
        Err(format!("无效的表达式部分: {}", input))
    }
}

/// 计算表达式结果
fn evaluate_expression(tokens: &[ExprToken]) -> Result<i64, String> {
    let mut values = Vec::new(); // 存储数字
    let mut operators = Vec::new(); // 存储操作符

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            ExprToken::Number(num) => {
                values.push(*num);
            }
            ExprToken::Operator(op) => {
                while let Some(prev_op) = operators.last() {
                    if *prev_op == 'x' || *prev_op == '/' {
                        let right = values.pop().ok_or("缺少右操作数")?;
                        let left = values.pop().ok_or("缺少左操作数")?;
                        let result = match prev_op {
                            'x' => left * right,
                            '/' => {
                                if right == 0 {
                                    return Err("除零错误".to_string());
                                }
                                left / right
                            }
                            _ => unreachable!(),
                        };
                        values.push(result);
                        operators.pop();
                    } else {
                        break;
                    }
                }
                operators.push(*op);
            }
            ExprToken::LeftParen => {
                // 找到匹配的右括号
                let mut j = i + 1;
                let mut paren_count = 1;
                while j < tokens.len() {
                    if let ExprToken::LeftParen = tokens[j] {
                        paren_count += 1;
                    } else if let ExprToken::RightParen = tokens[j] {
                        paren_count -= 1;
                        if paren_count == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                if paren_count != 0 {
                    return Err("括号不匹配".to_string());
                }

                // 递归计算括号内的表达式
                let sub_result = evaluate_expression(&tokens[i + 1..j])?;
                values.push(sub_result);

                // 跳过括号内的内容
                i = j;
            }
            ExprToken::RightParen => {
                return Err("多余的右括号".to_string());
            }
        }
        i += 1;
    }

    // 处理剩余的操作符
    while let Some(op) = operators.pop() {
        let right = values.pop().ok_or("缺少右操作数")?;
        let left = values.pop().ok_or("缺少左操作数")?;
        let result = match op {
            '+' => left + right,
            'x' => left * right,
            '/' => {
                if right == 0 {
                    return Err("除零错误".to_string());
                }
                left / right
            }
            _ => unreachable!(),
        };
        values.push(result);
    }

    values.pop().ok_or("表达式计算失败".to_string())
}

/// 这是一个简单的命令行工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 是否启用调试模式
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// 计算表达式 (--calc a + b / c)
    #[arg(
        short,
        long,
        num_args = 1.., // 接收至少一个参数
        value_parser = parse_expression_token,
    )]
    calc: Vec<ExprToken>, // 使用 Vec 存储解析后的表达式

    /// 是否启用 Gerrit 功能 (--gerrit)
    #[arg(short, long, default_value_t = false)]
    gerrit: bool,
}

fn get_padded_binary(num: i64) -> String {
    let binary_str = format!("{:b}", num);
    let len = binary_str.len();
    let padding = (4 - (len % 4)) % 4;
    "0".repeat(padding) + &binary_str
}

fn split_into_groups(s: &str) -> Vec<String> {
    s.chars()
        .collect::<Vec<_>>() // 将字符转换为 Vec<char>
        .chunks(4) // 按每四个字符分组
        .map(|chunk| chunk.iter().collect::<String>()) // 每组转换回字符串
        .collect()
}

fn print_binary_info(num: i64) {
    let padded_binary = get_padded_binary(num);
    let groups = split_into_groups(&padded_binary);
    let first_line = groups.join(" ");

    // 生成位索引
    let bit_positions: Vec<i64> = groups
        .iter()
        .enumerate()
        .map(|(i, _)| (padded_binary.len() as i64 - 4) - (i as i64 * 4))
        .collect();

    let second_line = bit_positions
        .iter()
        .map(|&x| format!("{:4}", x))
        .collect::<Vec<_>>()
        .join(" ");

    println!("二进制: ");
    println!("{}", first_line);
    println!("{}", second_line);
}

fn main() {
    let args: Args = Args::parse();

    if !args.calc.is_empty() {
        match evaluate_expression(&args.calc) {
            Ok(result) => {
                println!("十进制: {}", result);
                println!("十六进制: 0x{:X}", result);
                print_binary_info(result);
            }
            Err(err) => println!("错误: {}", err),
        }
    }

    // 如果启用了 --gerrit 参数，执行 Git 命令
    if args.gerrit {
        let cmd = "git push origin HEAD:refs/for/develop%r=zhaoqz";
        println!("{}", cmd);

        // 使用 spawn 和 Stdio::inherit 实现实时输出
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::inherit()) // 实时输出到终端
            .stderr(Stdio::inherit()) // 实时输出错误信息到终端
            .spawn()
            .expect("无法启动 Git 命令");

        // 等待命令完成
        let status = child.wait().expect("无法等待 Git 命令完成");

        if !status.success() {
            eprintln!("Git 命令执行失败");
        }
    }
}
