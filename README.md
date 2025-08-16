<div align="center">
  <img src="docs/images/prally.jpeg" alt="prally mascot" width="200" height="200" />

**A CLI ally for modern Git pull request workflows** ✨

[![Crates.io](https://img.shields.io/crates/v/prally.svg)](https://crates.io/crates/prally)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://github.com/capthiron/prally/actions/workflows/build.yml/badge.svg)](https://github.com/capthiron/prally/actions/workflows/build.yml)

*Because every developer deserves a supportive ally in their workflow* 💜
</div>

## 🌟 What is prally?

**prally** (pronounced "pr-ally") is your friendly neighborhood CLI tool that makes Git pull requests less of a chore
and more of a joy. Born from the belief that technology should be inclusive, accessible, and genuinely helpful, prally
streamlines your development workflow while keeping the human element at the center.

Whether you're a seasoned developer tired of writing the same PR descriptions over and over, or someone new to the field
looking for guidance, prally has your back. No judgment, just support. 🤗

> ⚡ **Weekend Warrior Alert!** The initial draft of prally was lovingly vibe-coded over a weekend, so expect the
> unexpected (but don't worry, we're not spreading fear here! 😄). While that scrappy foundation got us started, our
> focus
> is now squarely on polishing the experience and building a truly working and reliable CLI tool. Think of it as
> evolving
> from weekend prototype to production-ready ally! 🎯

## ✨ Features

### 🎯 MVP: Intelligent PR Description Generation

Transform your Git diffs and task descriptions into comprehensive, well-structured pull request descriptions using the
power of AI:

```bash
# Generate a description for your current branch
prally describe

# Use manual mode when you prefer hands-on control
prally describe --manual

# Get just the prompt for transparency and debugging
prally describe --prompt-only
```

### 🔮 Coming Soon

- **Automated Code Review**: Gentle, constructive feedback powered by LLMs
- **Changelog Generation**: Keep your project history organized and accessible
- **Workflow Chaining**: Combine actions for powerful, personalized workflows
- **Multi-platform Support**: GitHub, GitLab, and more

## 🚀 Quick Start

### Installation

**Homebrew (macOS/Linux):**

```bash
brew install prally
```

**Cargo (with Rust toolchain):**

```bash
cargo install prally
```

**Direct Binary Download:**
Download the latest release from our [releases page](https://github.com/your-username/prally/releases).

### Setup

Run the friendly setup wizard to get started:

```bash
prally setup
```

This will guide you through:

- 🔐 Securely storing your API tokens (GitHub, Jira, OpenAI, etc.)
- ⚙️ Configuring your preferred settings
- 🎨 Choosing your LLM provider and model

## 📚 Usage Examples

### Generate a PR Description

prally offers maximum flexibility in how you provide context for generating PR descriptions. You can use any combination
of these input methods to give the AI the best possible understanding of your changes.

#### **Basic Usage**

**Automatic Mode** (prally figures it out from your task tracker):

```bash
prally describe
```

**Git-Only Mode** (when no task tracker is configured):

```bash
prally describe
# Uses git diff as the primary context
```

**Manual Mode** (you're in complete control):

```bash
prally describe --manual
# Prompts you interactively for title and description
```

#### **Flexible Context Input Methods**

**1. Using Commit Messages as Context**

```bash
# Include commit messages from your current branch
prally describe --use-commits

# Perfect for branches with meaningful commit history
git log --oneline  # See what commits will be included
prally describe --use-commits
```

**2. Custom Instructions for the AI**

```bash
# Guide the AI's focus and tone
prally describe --instruction "Focus on security implications and breaking changes"

# Multiple instruction examples:
prally describe -i "Emphasize performance improvements"
prally describe -i "This is a hotfix - keep it concise"
prally describe -i "Include migration steps for database changes"
```

**3. Custom Context Information**

```bash
# Provide additional context about your changes
prally describe --custom-context "This fixes the memory leak reported in issue #456"

# More examples:
prally describe -c "Breaking change: requires Node.js 18+"
prally describe -c "Implements the new design system discussed in last week's meeting"
```

**4. Context from Files**

```bash
# Include content from documentation or specification files
prally describe --context-file design.md
prally describe -f requirements.txt
prally describe -f CHANGELOG.md

# Works with various file types (auto-detects format):
prally describe -f api-spec.json    # JSON files
prally describe -f feature.md       # Markdown files  
prally describe -f notes.txt        # Plain text files
```

**5. Piped Input from Other Commands**

```bash
# Pipe in context from other tools
cat meeting-notes.md | prally describe --from-stdin
echo "Focus on accessibility improvements" | prally describe --from-stdin

# Combine with other Unix tools:
grep -r "TODO" src/ | prally describe --from-stdin
curl -s https://api.example.com/spec | prally describe --from-stdin
```

**6. Manual Title and Description**

```bash
# Provide specific task information
prally describe --title "Add dark mode support" --desc "Implement dark mode toggle with system preference detection"

# Override auto-detected task info:
prally describe --title "Hotfix: Critical auth bug" --desc "Fixes authentication bypass vulnerability"
```

#### **Combining Input Methods**

The real power comes from combining multiple input sources:

```bash
# Comprehensive context for complex changes
prally describe \
  --use-commits \
  --instruction "Focus on backwards compatibility" \
  --custom-context "This addresses customer feedback from Q3 survey" \
  --context-file migration-guide.md

# Perfect for feature releases
prally describe \
  --context-file feature-spec.md \
  --custom-context "Implements design system v2.0" \
  --instruction "Highlight user-facing changes and migration path"

# Great for bug fixes
prally describe \
  --use-commits \
  --custom-context "Reproducer: curl -X POST /api/users (see issue #789)" \
  --instruction "Emphasize the root cause and prevention measures"

# Documentation-heavy changes  
cat design-decisions.md | prally describe \
  --from-stdin \
  --context-file architecture.md \
  --instruction "Focus on technical decision rationale"
```

#### **Development and Debugging**

**Transparency Mode** (see exactly what prally is working with):

```bash
prally describe --prompt-only
# Shows the complete context and prompt that will be sent to the AI
# Perfect for debugging or understanding how prally processes your input
```

**Iterative Refinement**:

```bash
# Start simple and add context as needed
prally describe --prompt-only  # See what context you have
prally describe --use-commits --prompt-only  # Add commit context
prally describe --use-commits -i "Focus on security" --prompt-only  # Add instruction
prally describe --use-commits -i "Focus on security"  # Generate final description
```

#### **Workflow Integration Examples**

**Pre-commit Hook Integration**:

```bash
# .git/hooks/pre-push
#!/bin/bash
if git log origin/main..HEAD --oneline | wc -l | grep -q "^[1-9]"; then
    echo "📝 Generating PR description preview..."
    prally describe --use-commits --instruction "Keep it concise for reviewer efficiency"
fi
```

**CI/CD Pipeline Integration**:

```bash
# Generate description for automated PR creation
prally describe \
  --use-commits \
  --context-file RELEASE_NOTES.md \
  --instruction "This is an automated release PR" > pr-description.md
```

**Daily Workflow Examples**:

```bash
# Quick feature branch
prally describe --use-commits

# Complex integration work  
prally describe \
  --context-file integration-plan.md \
  --custom-context "Phase 2 of the microservices migration" \
  --instruction "Highlight service dependencies and rollback plan"

# Emergency hotfix
prally describe \
  --custom-context "URGENT: Fixes production issue affecting user logins" \
  --instruction "Keep it brief, focus on impact and solution"

# Documentation update
echo "Updates based on user feedback and recent API changes" | \
  prally describe --from-stdin --instruction "Highlight what changed and why"
```

#### **Tips for Better Results**

1. **Layer Context Strategically**: Start with git diff (always included), add commits for narrative, then provide
   specific guidance through instructions and custom context.

2. **Use Instructions for Tone**: Guide the AI's focus and writing style:
    - `--instruction "Keep it technical for senior developers"`
    - `--instruction "Explain changes for junior team members"`
    - `--instruction "Emphasize business impact"`

3. **Provide Domain Context**: Use custom context for information not in your code:
    - `--custom-context "Addresses feedback from security audit"`
    - `--custom-context "Implements requirements from customer meeting"`

4. **Reference External Documentation**: Use context files for specifications, designs, or requirements that inform your
   changes.

5. **Combine Methods**: The most comprehensive descriptions come from combining multiple input sources that provide
   different perspectives on your changes.

````
