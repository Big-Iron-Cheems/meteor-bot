package common

// CommandBuilder struct to build a BaseCommand
type CommandBuilder struct {
	name        string
	description string
}

func NewCommandBuilder() *CommandBuilder {
	return &CommandBuilder{}
}

func (b *CommandBuilder) SetName(name string) *CommandBuilder {
	b.name = name
	return b
}

func (b *CommandBuilder) SetDescription(description string) *CommandBuilder {
	b.description = description
	return b
}

func (b *CommandBuilder) Build() *BaseCommand {
	return &BaseCommand{
		name:        b.name,
		description: b.description,
	}
}
