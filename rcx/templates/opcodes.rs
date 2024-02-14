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

{% endfor %}