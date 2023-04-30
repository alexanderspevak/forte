use std::collections::HashMap;

mod test;

pub type Value = i32;
pub type Result = std::result::Result<(), Error>;

pub struct Forth{
    _stack: Vec<Value>,
    _params:HashMap<String,(String,i32)>
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    DivisionByZero,
    StackUnderflow,
    UnknownWord,
    InvalidWord,
}

pub enum Op {
    Add,
    Subtract,
    Divide,
    Multiply,
    Dup,
    Drop,
    Swap,
    Over
}

impl TryFrom<&str>for Op {
    type Error=Error;

    fn try_from(word:&str)->std::result::Result<Op,Error>{
        match (word.to_lowercase()).as_str() {
            "+"=>return Ok(Op::Add),
            "-"=>return Ok(Op::Subtract),
            "*"=>return Ok(Op::Multiply),
            "/"=>return Ok(Op::Divide),
            "dup"=>return Ok(Op::Dup),
            "drop"=>return Ok(Op::Drop),
            "swap"=>return Ok(Op::Swap),
            "over"=>return Ok(Op::Over),
            _=>return Err(Error::UnknownWord)
        }
    }
}

impl Forth  {
    pub fn new() -> Forth {
        return Forth {
            _stack:Vec::new(),
            _params:HashMap::new()
        }
    }

    pub fn stack(&self) -> &[Value] {
        return &self._stack;
    }

    fn operate(&mut self,op:Op)->Result{
        match op {
            Op::Add=>{
                let num_1=self._stack.pop().ok_or(Error::StackUnderflow)?;
                let num_2=self._stack.pop().ok_or(Error::StackUnderflow)?;
                self._stack.push(num_1+num_2);
            },
            Op::Subtract=>{
                let num_1=self._stack.pop().ok_or(Error::StackUnderflow)?;
                let num_2=self._stack.pop().ok_or(Error::StackUnderflow)?;
                self._stack.push(num_2 -num_1);
            },
            Op::Multiply=>{
                let num_1=self._stack.pop().ok_or(Error::StackUnderflow)?;
                let num_2=self._stack.pop().ok_or(Error::StackUnderflow)?;
                self._stack.push(num_2 * num_1);
            },
            Op::Divide=>{
                let num_1=self._stack.pop().ok_or(Error::StackUnderflow)?;
                let num_2=self._stack.pop().ok_or(Error::StackUnderflow)?;
                
                if num_1 ==0 {
                    return Err(Error::DivisionByZero)
                }

                self._stack.push(num_2/num_1);
            },
            Op::Dup=>{
                let duplicated=self._stack.last().ok_or(Error::StackUnderflow)?;
                self._stack.push(*duplicated);
            }
            Op::Drop=>{
                self._stack.pop().ok_or(Error::StackUnderflow)?;
            },
            Op::Swap=>{
                let num_1=self._stack.pop().ok_or(Error::StackUnderflow)?;
                let num_2=self._stack.pop().ok_or(Error::StackUnderflow)?;
                self._stack.push(num_1);
                self._stack.push(num_2)
            },
            Op::Over=>{
                if self._stack.len()<2{
                    return Err(Error::StackUnderflow)
                }

                let over=self._stack.get(self._stack.len()-2).unwrap();
                self._stack.push(*over);
            }

            _=>return Err(Error::InvalidWord)
        }
        return Ok(());
    }


    fn get_stored_root_and_count(&self,key:&str)->(String,i32){
        let multi_words:Vec<&str>=key.split(" ").collect();
        let command_len=multi_words.len() as i32;

        let has_same_words= multi_words.windows(2).all(|w| w[0] == w[1]);

        if has_same_words  {
            match self._params.get(multi_words[0]){
                Some((key,child_count))=>{
                let (val, grand_child_count)= self.get_stored_root_and_count(key);
                return (String::from(val),command_len*child_count*grand_child_count);
                },
                None=>{
                    return (String::from(multi_words[0]), command_len);
                }
            }
        }
     
        return (String::from(key),1)
    }

    fn replace_params_with_values(&self, value:String)->String{
        let mut replaced_words=Vec::new();
        let value_arr=value.split(" ");

        for word in value_arr {
            match self._params.get(word){
                Some((word,count))=>{
                    for _ in 0..*count{
                        replaced_words.push(word.as_str())
                    }
                },
                None=>{
                    replaced_words.push(word)
                }
            }
        }
        return replaced_words.join(" ");
    }

    fn set_param(&mut self,key:&str,value:&str)->Result {
        if let Ok(_)=key.parse::<i32>(){
            return Err(Error::InvalidWord)
        }

        let (value,count)=self.get_stored_root_and_count(value);
        let value=self.replace_params_with_values(value);
        self._params.insert(String::from(key),(String::from(value),count));

        Ok(())
    }


    fn set_params(&mut self, input:&str)->Result {
        if let Some(ch)=input.chars().nth(0){
            if ch != ':'{
                return Err(Error::InvalidWord)
            }
        }

        if let Some(ch)=input.chars().nth(input.len()-1){
            if ch != ';'{
                return Err(Error::InvalidWord)
            }
        }

        let words:Vec<&str>=input[2..input.len()-2].split(" ").collect();
        let key=String::from(words[0]).to_lowercase();
        let value=Vec::from(&words[1..]).join(" ");

       return self.set_param(&key,&value)
    }

    fn sanitaze_words(&self,words:Vec<&str>)->Vec<String>{
        let mut sanitazed=Vec::new();
        for word in words {
            if word == "" { continue};
           if let Some(val)= self._params.get(word.to_lowercase().as_str()) {
                let (cloned,count)=val.clone();
                for _ in 0..count {
                    let new_words=cloned.split(" ");
                    for word in new_words {
                        sanitazed.push(String::from(word));
                    }
                }

                continue
           }

           sanitazed.push(String::from(word));
        }

        return sanitazed
    }

    fn execute_statement(&mut self, input: &str)->Result {
        let  words:Vec<&str>=input.split(" ").collect();
        let sanitazed_words: Vec<String>=self.sanitaze_words(words);
        
        for word in sanitazed_words {
            if let Ok(num)= word.parse::<i32>(){
                self._stack.push(num);
                continue
            }

            let try_from_result=Op::try_from(word.trim());
            if let Ok(op)=try_from_result{
                self.operate(op)?;
                continue;
            }

            if let Err(error)=try_from_result {
                return Err(error);
            }
        };
       Ok(())
    }

    fn split_to_statements(input:&str)->Vec<String>{
        let mut divided_inputs=Vec::new();
        let mut input = String::from(input);
        let mut is_user_params_input=false;
        
        while input.len()!=0{
            if !is_user_params_input {
                match input.find(':') {
                    Some(pos)=>{
                        divided_inputs.push(String::from(&input[..pos]));
                        input =String::from(&input[pos..]);
                        is_user_params_input=true;
                    },
                    None=>{
                        divided_inputs.push(input.clone());
                        input=String::from("");
                    }
                }
            }

            if is_user_params_input {
                match input.find(';') {
                    Some(pos)=>{
                        if pos== input.len()-1{
                            divided_inputs.push(input.clone());
                            return divided_inputs.into_iter().filter(|statement|statement!="").collect()
                        }

                        divided_inputs.push(String::from(&input[..=pos]));
                        input =String::from(&input[pos+2..]);
                    
                    },
                    None=>{
                        divided_inputs.push(input.clone());
                        return divided_inputs.into_iter().filter(|statement|statement!="").collect()

                    }
                }
            }
        }

        return divided_inputs.into_iter().filter(|statement|statement!="").collect();
        
    }

    fn evaluate_statement(&mut self, input: &str)->Result {
        if let Some(':')=input.chars().nth(0){
            return self.set_params(input)
        }

        return self.execute_statement(input)
    }

    pub fn eval(&mut self, input: &str) -> Result {
        let statements=Forth::split_to_statements(input);
        for input in statements {
            if let Err(error)=self.evaluate_statement(&input){
                return Err(error);
            }
        }

        Ok(())
    }
}



