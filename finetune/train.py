"""
Apophy Fine-Tune — QLoRA training from conversation memory.

Usage:
    uv run python finetune/train.py --dataset data/conversations.jsonl --output models/apophy-v1

Reads conversation fragments from SQLite, formats as instruction pairs,
and fine-tunes a base model using QLoRA (4-bit quantization).
"""

import argparse
import json
import sqlite3
from pathlib import Path


def load_fragments(db_path: str) -> list[dict]:
    """Load conversation fragments from Apophy SQLite database."""
    conn = sqlite3.connect(db_path)
    cursor = conn.execute(
        "SELECT id, session_id, content, created_at FROM fragments ORDER BY created_at"
    )
    fragments = [
        {
            "id": row[0],
            "session_id": row[1],
            "content": row[2],
            "created_at": row[3],
        }
        for row in cursor.fetchall()
    ]
    conn.close()
    return fragments


def fragments_to_dataset(fragments: list[dict]) -> list[dict]:
    """Convert memory fragments to instruction-response pairs."""
    pairs = []
    sessions: dict[str, list[dict]] = {}

    for f in fragments:
        sid = f["session_id"]
        if sid not in sessions:
            sessions[sid] = []
        sessions[sid].append(f)

    for sid, frags in sessions.items():
        for i in range(0, len(frags) - 1, 2):
            pairs.append(
                {
                    "instruction": frags[i]["content"],
                    "response": frags[i + 1]["content"] if i + 1 < len(frags) else "",
                    "session_id": sid,
                }
            )

    return pairs


def export_jsonl(pairs: list[dict], output_path: str) -> None:
    """Export dataset as JSONL for training."""
    path = Path(output_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w") as f:
        for pair in pairs:
            f.write(json.dumps(pair) + "\n")
    print(f"Exported {len(pairs)} training pairs to {output_path}")


def train(dataset_path: str, output_dir: str, base_model: str = "unsloth/gemma-2-2b") -> None:
    """Run QLoRA fine-tuning with Unsloth."""
    try:
        from unsloth import FastLanguageModel
        from trl import SFTTrainer
        from transformers import TrainingArguments
        from datasets import load_dataset
    except ImportError:
        print("Install dependencies: uv sync")
        print("Then: uv run python finetune/train.py")
        return

    print(f"Loading base model: {base_model}")
    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=base_model,
        max_seq_length=2048,
        load_in_4bit=True,
    )

    model = FastLanguageModel.get_peft_model(
        model,
        r=16,
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj"],
        lora_alpha=16,
        lora_dropout=0,
        use_gradient_checkpointing="unsloth",
    )

    dataset = load_dataset("json", data_files=dataset_path, split="train")

    trainer = SFTTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=dataset,
        args=TrainingArguments(
            per_device_train_batch_size=2,
            gradient_accumulation_steps=4,
            warmup_steps=5,
            max_steps=60,
            learning_rate=2e-4,
            output_dir=output_dir,
            logging_steps=1,
            fp16=True,
        ),
        dataset_text_field="instruction",
        max_seq_length=2048,
    )

    print("Training started...")
    trainer.train()

    print(f"Saving to {output_dir}")
    model.save_pretrained_gguf(output_dir, tokenizer, quantization_method="q4_k_m")
    print("Done. GGUF model ready for llm-router.")


def main() -> None:
    parser = argparse.ArgumentParser(description="Apophy QLoRA Fine-Tuning")
    sub = parser.add_subparsers(dest="command")

    # Export command
    export_cmd = sub.add_parser("export", help="Export fragments to JSONL")
    export_cmd.add_argument("--db", default="data/memory.db", help="SQLite database path")
    export_cmd.add_argument("--output", default="data/train.jsonl", help="Output JSONL path")

    # Train command
    train_cmd = sub.add_parser("train", help="Run QLoRA fine-tuning")
    train_cmd.add_argument("--dataset", default="data/train.jsonl", help="Training data JSONL")
    train_cmd.add_argument("--output", default="models/apophy-v1", help="Output model directory")
    train_cmd.add_argument("--base-model", default="unsloth/gemma-2-2b", help="Base model")

    args = parser.parse_args()

    if args.command == "export":
        fragments = load_fragments(args.db)
        pairs = fragments_to_dataset(fragments)
        export_jsonl(pairs, args.output)
    elif args.command == "train":
        train(args.dataset, args.output, args.base_model)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
