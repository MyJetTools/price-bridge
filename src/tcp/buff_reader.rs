
#[derive(Clone, Debug)]
pub struct BufferReader{
    buffer: [u8; 1024],
    start_indx: usize,
    end_indx: usize
}

impl BufferReader {

    pub fn new() -> BufferReader{
        BufferReader{
            buffer: [0; 1024],
            start_indx: 0,
            end_indx: 0
        }
    }

    pub fn write_to_buff(&mut self, data: Vec<u8>){

        if data.len() > self.get_amount_to_write() {
            panic!("Too long data to write in buff");
        }

        for byte in data{
            self.buffer[self.end_indx] = byte;
            self.end_indx = self.end_indx + 1;
        }
    }

    pub fn get_amount_to_write(&self) -> usize{
        return self.buffer.len() - self.end_indx -1;
    }

    pub fn get_next_package_if_exists(&mut self, sequence: &Vec<u8>) -> Option<Vec<u8>>{
        let next_indx = self.get_sequence_indx(sequence);

        if next_indx.is_none() {
            return None;
        }

        let result = self.buffer[self.start_indx..next_indx.unwrap() - sequence.len()].to_vec();
        self.start_indx = next_indx.unwrap();
        return Some(result);
    }

    pub fn compress(&mut self){
        if self.start_indx == self.end_indx
        {
            self.start_indx = 0;
            self.end_indx = 0;
            return;
        }

        let mut new_slice : [u8; 1024] = [0; 1024];
        let mut count = 0;

        for byte in &self.buffer[self.start_indx..self.end_indx] {
            new_slice[count] = byte.clone();
            count = count + 1;
        }

        self.buffer = new_slice;
        self.end_indx = self.end_indx - self.start_indx;
        self.start_indx = 0;
    }

    fn is_sequence_the_same(&self, target: &Vec<u8>, dest: &Vec<u8>) -> bool{
        if target.len() != dest.len() {
            return false;
        }

        for indx in 0..target.len() {
            if  target[indx] != dest[indx]{
                return false;
            }
        }

        return true;
    }

    fn get_sequence_indx(&self, sequence: &Vec<u8>) -> Option<usize>{

        if sequence.len() == 0 {
            panic!("Lenght of sequence cant be 0")
        }

        if self.end_indx < sequence.len() {
            return None;
        }

        let range_to_iter_without_sequence: usize = self.end_indx - sequence.len();

        for indx in self.start_indx..range_to_iter_without_sequence  {
            if self.buffer[indx] == sequence[0] {
                if self.is_sequence_the_same(&self.buffer[indx..indx+sequence.len()].to_vec(), sequence) {
                    return Some(indx + sequence.len());
                }
            }
        }

        return None;

    }
}


#[cfg(test)]
mod tests {

    use super::BufferReader;

    #[test]
    fn test_buff_sequence_reed() {
        let mut buff = BufferReader::new();

        let splitter = "/n".as_bytes().to_vec();

        let first_data = "First message/n".as_bytes().to_vec();
        let second_data = "Second mes".as_bytes().to_vec();
        let third_data = "sage/n".as_bytes().to_vec();
        let four_data = "First me".as_bytes().to_vec();

        buff.write_to_buff(first_data.clone());
        buff.write_to_buff(second_data.clone());
        buff.write_to_buff(third_data.clone());
        buff.write_to_buff(four_data.clone());

        let first_mess_from_socket = buff.get_next_package_if_exists(&splitter);
        buff.compress();
        let second_mess_from_socket = buff.get_next_package_if_exists(&splitter);
        buff.compress();
        let third_mess_from_socket = buff.get_next_package_if_exists(&splitter);
        buff.compress();

        assert_eq!(first_mess_from_socket.is_some(), true);
        assert_eq!("First message".as_bytes(), first_mess_from_socket.unwrap());
        assert_eq!(second_mess_from_socket.is_some(), true);
        assert_eq!("Second message".as_bytes(), second_mess_from_socket.unwrap());
        assert_eq!(third_mess_from_socket.is_some(), false);
    }
}
