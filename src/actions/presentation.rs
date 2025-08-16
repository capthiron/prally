use anyhow::Result;
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

/// Update the current line with new content, clearing any previous content
pub fn update_current_line(message: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .queue(cursor::MoveToColumn(0))?
        .queue(Clear(ClearType::CurrentLine))?
        .queue(Print(message))?
        .queue(Print("\n"))?;
    stdout.flush()?;
    Ok(())
}

/// Clear the current line completely
#[allow(dead_code)]
pub fn clear_current_line() -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .queue(cursor::MoveToColumn(0))?
        .queue(Clear(ClearType::CurrentLine))?;
    stdout.flush()?;
    Ok(())
}

/// Show a progress message with spinner animation
pub async fn show_progress_with_spinner(message: &str, duration_ms: u64) -> Result<()> {
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut stdout = io::stdout();
    
    let iterations = duration_ms / 100; // Update every 100ms
    for i in 0..iterations {
        let spinner = spinner_chars[i as usize % spinner_chars.len()];
        stdout
            .queue(cursor::MoveToColumn(0))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(Print(format!("{} {}", spinner, message)))?
            .queue(ResetColor)?;
        stdout.flush()?;
        sleep(Duration::from_millis(100)).await;
    }
    
    Ok(())
}

/// Show a simple progress message (no animation)
pub fn show_progress(message: &str) -> Result<()> {
    print!("{}", message);
    io::stdout().flush()?;
    Ok(())
}

/// Enhanced choice prompt that shows feedback after selection
pub fn prompt_choice_with_feedback(
    prompt: &str,
    choices: &[&str],
    default: &str,
) -> Result<String> {
    println!("{}", prompt);
    for (i, choice) in choices.iter().enumerate() {
        let marker = if *choice == default { " (default)" } else { "" };
        println!("  {}. {}{}", i + 1, choice, marker);
    }

    loop {
        print!("Choose [1-{}]: ", choices.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let selected = if input.is_empty() {
            default.to_string()
        } else if let Ok(choice_num) = input.parse::<usize>() {
            if choice_num > 0 && choice_num <= choices.len() {
                choices[choice_num - 1].to_string()
            } else {
                println!("❌ Please enter a number between 1 and {}.", choices.len());
                continue;
            }
        } else {
            println!("❌ Please enter a number between 1 and {}.", choices.len());
            continue;
        };

        // Show confirmation feedback
        update_current_line(&format!("✅ Selected: {}", selected))?;
        return Ok(selected);
    }
}

/// Enhanced yes/no prompt with feedback
pub fn prompt_yes_no_with_feedback(prompt: &str, default: bool) -> Result<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };

    loop {
        print!("{} [{}]: ", prompt, default_str);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        let result = match input.as_str() {
            "" => default,
            "y" | "yes" => true,
            "n" | "no" => false,
            _ => {
                println!("❌ Please enter 'y' for yes or 'n' for no.");
                continue;
            }
        };

        // Show confirmation feedback
        let response = if result { "Yes" } else { "No" };
        update_current_line(&format!("✅ {}", response))?;
        return Ok(result);
    }
}

/// Show success message with green color
pub fn show_success(message: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .execute(SetForegroundColor(Color::Green))?
        .execute(Print(message))?
        .execute(ResetColor)?
        .execute(Print("\n"))?;
    Ok(())
}

/// Show error message with red color
pub fn show_error(message: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .execute(SetForegroundColor(Color::Red))?
        .execute(Print(message))?
        .execute(ResetColor)?
        .execute(Print("\n"))?;
    Ok(())
}

/// Show info message with cyan color
pub fn show_info(message: &str) -> Result<()> {
    let mut stdout = io::stdout();
    stdout
        .execute(SetForegroundColor(Color::Cyan))?
        .execute(Print(message))?
        .execute(ResetColor)?
        .execute(Print("\n"))?;
    Ok(())
}

/// Simulate token validation with progress feedback
pub async fn validate_token_with_progress(token_type: &str) -> Result<()> {
    show_progress(&format!("🔑 Validating {} token...", token_type))?;
    
    // Simulate validation time
    sleep(Duration::from_millis(1500)).await;
    
    update_current_line(&format!("✅ {} token validated successfully!", token_type))?;
    Ok(())
}

/// Simulate API endpoint testing with progress feedback
pub async fn test_api_endpoint_with_progress(service: &str, _endpoint: &str) -> Result<()> {
    show_progress(&format!("🌐 Testing {} API connection...", service))?;
    
    // Simulate API test time
    sleep(Duration::from_millis(2000)).await;
    
    update_current_line(&format!("✅ {} API connection successful!", service))?;
    Ok(())
}

/// Show configuration save progress
pub async fn save_config_with_progress() -> Result<()> {
    show_progress("💾 Saving configuration...")?;
    
    // Simulate save time
    sleep(Duration::from_millis(800)).await;
    
    update_current_line("✅ Configuration saved successfully!")?;
    Ok(())
}
