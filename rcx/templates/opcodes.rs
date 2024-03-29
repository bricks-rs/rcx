// This file is automatically generated

{% for opcode in opcodes %}
#[doc=r#"{{ opcode.description }}"#]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct {{ opcode.name }} {
    {% for param in opcode.request.params %}
    pub {{ param.name }}: {{ param.ty }},
    {% endfor %}
}

impl Opcode for {{ opcode.name }} {
    fn request_opcode(&self) -> u8 {
        {{ opcode.request.opcode|hex }}
    }

    fn response_opcode(&self) -> Option<u8> {
    {% if let Some(response) = opcode.response %}
        Some({{ response.opcode|hex}})
    {% else %}
        None
    {% endif %}
    }

    fn serialise(&self, buf: &mut [u8]) -> Result<usize> {
        #[allow(unused_mut)]
        let mut cursor = Cursor::new(buf);
        {% for param in opcode.request.params %}
        self.{{ param.name }}.write_param(&mut cursor)?;
        {% endfor %}
        Ok(cursor.position().try_into()?)
    }

    #[allow(unreachable_code, unused_variables, clippy::diverging_sub_expression)]
    fn disasm(bin: &[u8], pc: &mut usize) -> Result<Self> {
        {% for param in opcode.request.params %}
        let {{ param.name }} =
            {% if param.ty == "Vec<u8>" %}
            unimplemented!();
            {% else %}
            <{{ param.ty }} as DisasmParam>::disasm_param(bin, pc)?;
            {% endif %}
        {% endfor %}

        Ok(Self {
            {% for param in opcode.request.params %}
            {{ param.name }},
            {% endfor %}
        })
    }
}


{% if let Some(response) = opcode.response %}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct {{ opcode.name }}Response {
    {% for param in response.params %}
    pub {{ param.name }}: {{ param.ty }},
    {% endfor %}
}

impl {{ opcode.name }}Response {
    pub fn deserialise(buf: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(buf);

        // skip header bytes
        loop {
            let mut byte=[0];
            cursor.read_exact(&mut byte)?;
            if !is_header(byte[0]) {
                cursor.seek(SeekFrom::Current(-1))?;
                break;
            }
        }

        // read & verify opcode
        let opcode = u8::read_param(&mut cursor)?;

        // not every opcode will write to the checksum
        #[allow(unused_mut)]
        let mut checksum = opcode;

        // parse out fields
        {% for param in response.params %}
        let {{ param.name }} =
            <{{ param.ty }} as ReadParam>::read_param(&mut cursor)?;
        {{ param.name }}.add_to_checksum(&mut checksum);
        {% endfor %}

        // validate checksum
        let pkt_checksum = u8::read_param(&mut cursor)?;

        if checksum == pkt_checksum {
            Ok(Self {
                {% for param in response.params %}
                {{ param.name }},
                {% endfor %}
            })
        } else {
            Err(Error::Checksum)
        }
    }
}
{% endif %}

impl Display for {{ opcode.name }} {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "{{ opcode.name }}")?;
        {% for param in opcode.request.params %}
        write!(fmt, " {{ param.name }}={:02x?}", self.{{ param.name }})?;
        {% endfor %}
        Ok(())
    }
}

{% endfor %}

pub fn parse_opcode(bin: &[u8], pc: &mut usize) -> Result<Opcodes> {
    let code = read_byte(bin, pc)?;
    match code {
        {% for opcode in opcodes %}
        {{ opcode.request.opcode|hex }} =>
            Ok(Opcodes::{{ opcode.name }}({{ opcode.name }}::disasm(bin, pc)?)),
        {% endfor %}
        other => Err(Error::InvalidOpcode(other)),
    }
}

#[derive(Debug)]
pub enum Opcodes {
    {% for opcode in opcodes %}
    {{ opcode.name }}({{ opcode.name }}),
    {% endfor %}
}

impl Display for Opcodes {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            {% for opcode in opcodes %}
            Self::{{ opcode.name }}(code) => write!(fmt, "{code}"),
            {% endfor %}
        }
    }
}

impl Opcode for Opcodes {
    fn request_opcode(&self) -> u8 {
        match self {
            {% for opcode in opcodes %}
            Self::{{ opcode.name }}(code) => code.request_opcode(),
            {% endfor %}
        }
    }
    fn response_opcode(&self) -> Option<u8> {
        todo!()
    }
    fn serialise(&self, buf: &mut [u8]) -> Result<usize> {
        match self {
            {% for opcode in opcodes %}
            Self::{{ opcode.name }}(code) => code.serialise(buf),
            {% endfor %}
        }
    }
    fn disasm(_bin: &[u8], _pc: &mut usize) -> Result<Self> {
        todo!()
    }
}